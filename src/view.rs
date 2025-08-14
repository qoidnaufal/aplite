use std::cell::{RefCell, Ref, RefMut};
use std::rc::{Rc, Weak};

use aplite_renderer::{Renderer, Shape};
use aplite_reactive::*;
use aplite_storage::{Entity, entity};
use aplite_types::{
    CornerRadius,
    Rgba,
    Paint,
    Size,
    Rect,
};

use crate::widget::{Widget, WidgetId};
use crate::state::WidgetState;
use crate::context::layout::*;
use crate::context::DIRTY;

entity! { pub ViewId }

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

impl<T: Widget + 'static> IntoView for T {
    fn into_view(self) -> View {
        View::new(self)
    }
}

impl Widget for Box<&mut dyn IntoView> {
    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn IntoView>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn IntoView>>> {
        self.as_mut().children_mut()
    }
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    inner: Box<dyn Widget>
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            inner: Box::new(widget),
        }
    }
}

impl Widget for View {
    fn node(&self) -> ViewNode {
        self.inner.node()
    }

    fn id(&self) -> WidgetId {
        self.inner.id()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn IntoView>>> {
        self.inner.children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn IntoView>>> {
        self.inner.children_mut()
    }
}

impl Widget for Box<dyn IntoView> {
    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn IntoView>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn IntoView>>> {
        self.as_mut().children_mut()
    }
}

impl std::fmt::Debug for Box<dyn Widget> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.node().borrow().name;
        let name = name.is_empty()
            .then_some(std::any::type_name::<Self>())
            .unwrap_or(name);

        f.debug_struct(name)
            .field("id", &self.id())
            .field("children", &self.children_ref().unwrap_or(&vec![]))
            .finish()
    }
}

impl std::fmt::Debug for Box<dyn IntoView> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn IntoView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.node().borrow().name;
        let name = name.is_empty()
            .then_some(std::any::type_name::<Self>())
            .unwrap_or(name);

        f.debug_struct(name)
            .field("id", &self.id())
            .field("children", &self.children_ref().unwrap_or(&vec![]))
            .finish()
        
    }
}

impl<T: IntoView + Sized> Render for T {}

pub(crate) trait Render: IntoView + Sized {
    fn render(&self, renderer: &mut Renderer) {
        self.draw(renderer);
        if let Some(children) = self.children_ref() {
            children.iter().for_each(|w| w.render(renderer));
        }
    }

    fn calculate_size(&mut self) -> Size {
        if let Some(children) = self.children_mut() {
        }
        todo!()
    }

    // should include calculating the size too here
    fn calculate_layout(&mut self, cx: &mut LayoutCx) {
        self.layout(cx);

        let mut this_cx = LayoutCx::new(self);

        if let Some(children) = self.children_mut() {
            children.iter_mut()
                .for_each(|child| child.calculate_layout(&mut this_cx));
        }
    }

    fn find(&self, id: &WidgetId) -> Option<Box<&dyn IntoView>> {
        if self.id() == id {
            return Some(Box::new(self))
        }

        self.children_ref().and_then(|vec| {
            vec.iter().find_map(|w| w.find(id))
        })
    }

    fn find_mut(&mut self, id: &WidgetId) -> Option<Box<&mut dyn IntoView>> {
        if self.id() == id {
            return Some(Box::new(self))
        }
        self.children_mut().and_then(|vec| {
            vec.iter_mut().find_map(|w| w.find_mut(id))
        })
    }

    fn parent_mut(&mut self, id: &WidgetId) -> Option<Box<&mut dyn IntoView>> {
        if let Some(children) = self.children_ref()
            && children
                .iter()
                .any(|w| w.id() == id)
        {
            return Some(Box::new(self))
        }

        self.children_mut()
            .and_then(|vec| {
                vec.iter_mut()
                    .find_map(|w| w.parent_mut(id))
            })
    }

    fn remove(&mut self, id: &WidgetId) -> Option<Box<dyn IntoView>> {
        self.parent_mut(id)
            .and_then(|mut parent| {
                parent.children_mut()
                    .and_then(|children| {
                        children.iter()
                            .position(|w| w.id() == id)
                            .map(|index| children.remove(index))
                    })
            })
    }

    fn insert<T: Widget + 'static>(&mut self, parent: &WidgetId, widget: T) {
        if let Some(mut p) = self.find_mut(parent)
            && let Some(vec) = p.children_mut()
        {
            vec.push(Box::new(widget));
        }
    }
}

#[derive(Clone, Debug)]
pub struct ViewNode(pub(crate) Rc<RefCell<WidgetState>>);

impl ViewNode {
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(WidgetState::new()));

        Self(state)
    }

    pub(crate) fn window(rect: Rect) -> Self {
        let state = Rc::new(RefCell::new(WidgetState::window(rect)));

        Self(state)
    }

    pub fn node_ref(&self) -> ViewNodeRef {
        let dirty = DIRTY.get().unwrap();
        ViewNodeRef(Rc::downgrade(&self.0), dirty.write_only())
    }

    #[inline(always)]
    pub(crate) fn borrow(&self) -> Ref<'_, WidgetState> {
        self.0.borrow()
    }

    #[inline(always)]
    pub(crate) fn borrow_mut(&self) -> RefMut<'_, WidgetState> {
        self.0.borrow_mut()
    }

    pub fn with_name(self, name: &'static str) -> Self {
        self.0.borrow_mut().set_name(name);
        self
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, size: impl Into<Size>) -> Self {
        self.0.borrow_mut().set_size(size);
        self
    }

    pub fn with_min_width(self, val: f32) -> Self {
        self.0.borrow_mut().min_width = Some(val);
        self
    }

    pub fn with_max_width(self, val: f32) -> Self {
        self.0.borrow_mut().max_width = Some(val);
        self
    }

    pub fn with_min_height(self, val: f32) -> Self {
        self.0.borrow_mut().min_height = Some(val);
        self
    }

    pub fn with_max_height(self, val: f32) -> Self {
        self.0.borrow_mut().max_height = Some(val);
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(self, paint: impl Into<Paint>) -> Self {
        self.0.borrow_mut().background_paint = paint.into();
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(self, color: impl Into<Paint>) -> Self {
        self.0.borrow_mut().border_paint = color.into();
        self
    }

    pub fn with_stroke_width(self, val: f32) -> Self {
        self.0.borrow_mut().border_width = val;
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        self.0.borrow_mut().shape = shape;
        self
    }

    pub fn with_rotation_deg(self, deg: f32) -> Self {
        self.with_rotation_rad(deg.to_radians())
    }

    pub fn with_rotation_rad(self, rad: f32) -> Self {
        self.0.borrow_mut().set_rotation_rad(rad);
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        self.0.borrow_mut().set_corner_radius(val);
        self
    }

    pub fn with_horizontal_align(self, align_h: AlignH) -> Self {
        self.0.borrow_mut().align_h = align_h;
        self
    }

    pub fn with_vertical_align(self, align_v: AlignV) -> Self {
        self.0.borrow_mut().align_v = align_v;
        self
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        self.0.borrow_mut().orientation = orientation;
        self
    }

    pub fn hoverable(self) -> Self {
        self.0.borrow_mut().hoverable = true;
        self
    }
}

#[derive(Clone, Debug)]
pub struct ViewNodeRef(Weak<RefCell<WidgetState>>, SignalWrite<bool>);

impl ViewNodeRef {
    pub(crate) fn upgrade(&self) -> Option<ViewNode> {
        self.0.upgrade().map(|rc| ViewNode(rc))
    }

    pub fn set_color(&self, color: Rgba<u8>) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().background_paint = color.into();
            self.1.set(true);
        }
    }

    pub fn set_shape(&self, shape: Shape) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().shape = shape;
            self.1.set(true);
        }
    }

    pub fn set_rotation_deg(&self, deg: f32) {
        self.set_rotation_rad(deg.to_radians());
    }

    pub fn set_rotation_rad(&self, rad: f32) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().rotation = rad;
            self.1.set(true);
        }
    }

    pub fn set_spacing(&self, val: f32) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().spacing = val;
            self.1.set(true);
        }
    }

    pub fn hide(&self, val: bool) {
        if let Some(node) = self.upgrade() {
            node.0.borrow_mut().hide = val;
            self.1.set(true);
        }
    }
}
