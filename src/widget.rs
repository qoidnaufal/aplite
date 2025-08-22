use std::cell::RefCell;

use aplite_renderer::{Shape, Scene};
use aplite_types::{Rgba, CornerRadius, Size, Rect};
use aplite_storage::U64Map;

use crate::state::{WidgetId, NodeRef, AspectRatio};
use crate::layout::*;
use crate::view::IntoView;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

pub struct ChildrenRef<'a>(&'a Vec<Box<dyn Widget>>);

impl<'a> From<&'a Vec<Box<dyn Widget>>> for ChildrenRef<'a> {
    fn from(value: &'a Vec<Box<dyn Widget>>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ChildrenRef<'_> {
    type Target = Vec<Box<dyn Widget>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::fmt::Debug for ChildrenRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(self.0)
            .finish()
    }
}

pub struct ChildrenMut<'a>(&'a mut Vec<Box<dyn Widget>>);

impl<'a> From<&'a mut Vec<Box<dyn Widget>>> for ChildrenMut<'a> {
    fn from(value: &'a mut Vec<Box<dyn Widget>>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ChildrenMut<'_> {
    type Target = Vec<Box<dyn Widget>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::ops::DerefMut for ChildrenMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl std::fmt::Debug for ChildrenMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(self.0)
            .finish()
    }
}

/// main building block to create a renderable component
pub trait Widget {
    fn node_ref(&self) -> NodeRef;

    fn id(&self) -> WidgetId {
        self.node_ref().id()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        None
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        None
    }

    fn draw(&self, scene: &mut Scene) {
        let node = self.node_ref().upgrade();

        if !node.borrow().flag.is_hidden() {
            if node.borrow().flag.is_dirty() {
                let state = node.borrow();
                scene.draw(
                    &state.rect,
                    state.transform,
                    state.background_paint.as_paint_ref(),
                    state.border_paint.as_paint_ref(),
                    state.border_width.max(5.0),
                    state.shape,
                    &state.corner_radius,
                );
                drop(state);
                node.borrow_mut().flag.set_dirty(false);
            } else {
                scene.next_frame();
            }

            if let Some(children) = self.children_ref() {
                children
                    .iter()
                    .for_each(|child| child.draw(scene));
            }
        }
    }

    fn layout(&self, cx: &mut LayoutCx) -> bool {
        let node = self.node_ref().upgrade();
        if node.borrow().flag.is_hidden() { return false }

        let size = node.borrow().rect.size();
        let mut this = node.borrow_mut();

        match cx.rules.orientation {
            Orientation::Vertical => {
                match cx.rules.align_h {
                    AlignH::Left | AlignH::Right => this.rect.x = cx.next_pos.x,
                    AlignH::Center => this.rect.x = cx.next_pos.x - size.width / 2.,
                }

                this.rect.y = cx.next_pos.y;
                cx.next_pos.y += cx.rules.spacing as f32 + size.height;
            },
            Orientation::Horizontal => {
                match cx.rules.align_v {
                    AlignV::Top | AlignV::Bottom => this.rect.y = cx.next_pos.y,
                    AlignV::Middle => this.rect.y = cx.next_pos.y - size.height / 2.,
                }

                this.rect.x = cx.next_pos.x;
                cx.next_pos.x += cx.rules.spacing as f32 + size.width;
            },
        }

        this.flag.set_dirty(true);

        true
    }
}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn child(mut self, child: impl IntoView + 'static) -> Self {
        if let Some(mut children) = self.children_mut() {
            // let child_size = child.node().borrow().rect.size();
            children.push(Box::new(child));
        }
        self
    }

    fn on<F>(self, event: WidgetEvent, f: F) -> Self
    where
        F: FnMut() + 'static,
    {
        CALLBACKS.with(|cell| {
            let mut storage = cell.borrow_mut();
            let callbacks = storage.entry(self.id()).or_default();
            callbacks.insert(event, Box::new(f));
        });

        self
    }

    fn image_aspect_ratio(self, val: AspectRatio) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .image_aspect_ratio = val;

        self
    }

    fn color(self, color: Rgba) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .background_paint = color.into();

        self
    }

    fn border_color(self, color: Rgba) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .border_paint = color.into();

        self
    }

    fn hover_color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn click_color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn border_width(self, val: f32) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .border_width = val;

        self
    }

    fn corner_radius(self, corner_radius: CornerRadius) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .corner_radius = corner_radius;

        self
    }

    fn shape(self, shape: Shape) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .shape = shape;

        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .rect.set_size(size.into());

        self
    }

    fn dragable(self) -> Self {
        let node = self.node_ref().upgrade();
        let mut node = node.borrow_mut();
        node.flag.set_dragable(true);
        node.flag.set_hoverable(true);

        self
    }

    fn spacing(self, val: u8) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .spacing = val;

        self
    }

    fn padding(self, padding: Padding) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .padding = padding;

        self
    }

    fn min_width(self, val: f32) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .min_width = Some(val);

        self
    }

    fn min_height(self, val: f32) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .min_height = Some(val);

        self
    }

    fn max_width(self, val: f32) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .max_width = Some(val);

        self
    }

    fn max_height(self, val: f32) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .max_height = Some(val);

        self
    }

    fn align_h(self, align_h: AlignH) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .align_h = align_h;
        self
    }

    fn align_v(self, align_v: AlignV) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .align_v = align_v;
        self
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

impl Widget for Box<dyn Widget> {
    fn node_ref(&self) -> NodeRef {
        self.as_ref().node_ref()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        self.as_mut().children_mut()
    }
}

impl Widget for Box<&mut dyn Widget> {
    fn node_ref(&self) -> NodeRef {
        self.as_ref().node_ref()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        self.as_mut().children_mut()
    }
}

impl Widget for *const dyn Widget {
    fn node_ref(&self) -> NodeRef {
        unsafe {
            self.as_ref().unwrap().node_ref()
        }
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        unsafe {
            self.as_ref().and_then(|w| w.children_ref())
        }
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        unsafe {
            self.cast_mut()
                .as_mut()
                .and_then(|w| w.children_mut())
        }
    }
}

// -------------------------------------

thread_local! {
    pub(crate) static CALLBACKS: RefCell<U64Map<WidgetId, CallbackStore>>
        = RefCell::new(Default::default());
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetEvent {
    Hover,
    LeftClick,
    RightClick,
    Drag,
    Input,
}

pub(crate) struct CallbackStore(Box<[Option<Box<dyn FnMut()>>; 5]>);

impl Default for CallbackStore {
    fn default() -> Self {
        Self(Box::new([None, None, None, None, None]))
    }
}

impl CallbackStore {
    pub(crate) fn insert(
        &mut self,
        event: WidgetEvent,
        callback: Box<dyn FnMut()>,
    ) -> Option<Box<dyn FnMut()>> {
        self.0[event as usize].replace(callback)
    }

    pub(crate) fn get_mut(&mut self, event: WidgetEvent) -> Option<&mut Box<dyn FnMut()>> {
        self.0[event as usize].as_mut()
    }
}

// -------------------------------------

pub(crate) struct WindowWidget {
    node: NodeRef,
    children: Vec<Box<dyn Widget>>,
}

impl WindowWidget {
    pub(crate) fn new(rect: Rect) -> Self {
        Self {
            node: NodeRef::window(rect),
            children: Vec::new(),
        }
    }
}

impl Widget for WindowWidget {
    fn node_ref(&self) -> NodeRef {
        self.node.clone()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        Some((&self.children).into())
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        Some((&mut self.children).into())
    }
}

// -------------------------------------

pub struct CircleWidget {
    node: NodeRef,
}

impl CircleWidget {
    pub fn new() -> Self {
        Self {
            node: NodeRef::new()
                .with_name("Circle")
                .with_stroke_width(5.)
                .with_shape(Shape::Circle)
                .with_size((100., 100.)),
        }
    }
}

impl Widget for CircleWidget {
    fn node_ref(&self) -> NodeRef {
        self.node.clone()
    }
}
