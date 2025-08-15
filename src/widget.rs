use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use aplite_renderer::{Shape, Renderer};
use aplite_types::{Rgba, CornerRadius, Size, Rect};
use aplite_storage::U64Map;

use crate::state::{ViewNode, ViewNodeRef, AspectRatio};
use crate::context::layout::*;
use crate::view::IntoView;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

#[derive(Clone, Copy, Eq, PartialOrd, Ord)]
pub struct WidgetId(u64);

impl WidgetId {
    pub(crate) fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Relaxed))
    }
}

impl PartialEq<Self> for WidgetId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<&Self> for WidgetId {
    fn eq(&self, other: &&Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WidgetId({})", self.0)
    }
}

impl std::hash::Hash for WidgetId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

/// main building block to create a renderable component
pub trait Widget {
    fn id(&self) -> WidgetId;

    fn node(&self) -> ViewNode;

    fn node_ref(&self) -> ViewNodeRef {
        self.node().node_ref()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        None
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        None
    }

    fn draw(&self, renderer: &mut Renderer) -> bool {
        let node = self.node();
        let hide = node.borrow().hide;

        if !hide {
            let state = node.borrow();
            let scene = renderer.scene();
            let size = scene.size();

            let transform = state.get_transform(size);
            let rotation = state.rotation;
            let background_paint = state.background_paint.as_paint_ref();
            let border_paint = state.border_paint.as_paint_ref();
            let shape = state.shape;
            let border_width = (state.border_width == 0.0)
                .then_some(5.0 / size.width)
                .unwrap_or(state.border_width / size.width);

            scene.draw(
                transform,
                rotation,
                background_paint,
                border_paint,
                border_width,
                shape
            );
        }

        !hide
    }

    fn layout(&self, cx: &mut LayoutCx) -> bool {
        let node = self.node();

        if node.borrow().hide { return false }

        let size = node.borrow().rect.size();

        let mut this = node.borrow_mut();

        match cx.rules.orientation {
            Orientation::Vertical => {
                match cx.rules.align_h {
                    AlignH::Left | AlignH::Right => this.rect.x = cx.next_pos.x,
                    AlignH::Center => this.rect.x = cx.next_pos.x - size.width / 2.,
                }

                this.rect.y = cx.next_pos.y;
                cx.next_pos.y += cx.rules.spacing + size.height;
            },
            Orientation::Horizontal => {
                match cx.rules.align_v {
                    AlignV::Top | AlignV::Bottom => this.rect.y = cx.next_pos.y,
                    AlignV::Middle => this.rect.y = cx.next_pos.y - size.height / 2.,
                }

                this.rect.x = cx.next_pos.x;
                cx.next_pos.x += cx.rules.spacing + size.width;
            },
        }

        true
    }
}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn child(mut self, child: impl IntoView + 'static) -> Self {
        if let Some(children) = self.children_mut() {
            // let child_size = child.node().borrow().rect.size();
            children.push(Box::new(child));
        }
        self
    }

    fn and(self, sibling: impl IntoView + 'static) -> Self {
        let _ = sibling;
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
        self.node().borrow_mut().image_aspect_ratio = val;
        self
    }

    fn color(self, color: Rgba<u8>) -> Self {
        self.node().borrow_mut().background_paint = color.into();
        self
    }

    fn border_color(self, color: Rgba<u8>) -> Self {
        self.node().borrow_mut().border_paint = color.into();
        self
    }

    fn hover_color(self, color: Rgba<u8>) -> Self {
        let _ = color;
        self
    }

    fn click_color(self, color: Rgba<u8>) -> Self {
        let _ = color;
        self
    }

    fn border_width(self, val: f32) -> Self {
        self.node().borrow_mut().border_width = val;
        self
    }

    fn corners(self, corners: CornerRadius) -> Self {
        self.node().borrow_mut().corner_radius = corners;
        self
    }

    fn shape(self, shape: Shape) -> Self {
        self.node().borrow_mut().shape = shape;
        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        self.node().borrow_mut().rect.set_size(size.into());
        self
    }

    fn dragable(self) -> Self {
        let node = self.node();
        let mut node = node.borrow_mut();
        node.dragable = true;
        node.hoverable = true;

        self
    }

    fn spacing(self, val: f32) -> Self {
        self.node().borrow_mut().spacing = val;
        self
    }

    fn padding(self, padding: Padding) -> Self {
        self.node().borrow_mut().padding = padding;
        self
    }

    fn min_width(self, val: f32) -> Self {
        let node = self.node();
        node.borrow_mut().min_width = Some(val);

        let state = node.borrow();
        let min_width = state.min_width;
        let min_height = state.min_height;
        let max_width = state.max_width;
        let max_height = state.max_height;
        let current_size = state.rect.size();

        drop(state);

        let new_size = current_size
            .adjust_on_min_constraints(min_width, min_height)
            .adjust_on_max_constraints(max_width, max_height);

        node.borrow_mut().rect.set_size(new_size);

        self
    }

    fn min_height(self, val: f32) -> Self {
        let node = self.node();
        node.borrow_mut().min_height = Some(val);

        let state = node.borrow();
        let min_width = state.min_width;
        let min_height = state.min_height;
        let max_width = state.max_width;
        let max_height = state.max_height;
        let current_size = state.rect.size();

        drop(state);

        let new_size = current_size
            .adjust_on_min_constraints(min_width, min_height)
            .adjust_on_max_constraints(max_width, max_height);

        node.borrow_mut().rect.set_size(new_size);

        self
    }

    fn max_width(self, val: f32) -> Self {
        let node = self.node();
        node.borrow_mut().max_width = Some(val);

        let state = node.borrow();
        let min_width = state.min_width;
        let min_height = state.min_height;
        let max_width = state.max_width;
        let max_height = state.max_height;
        let current_size = state.rect.size();

        drop(state);

        let new_size = current_size
            .adjust_on_min_constraints(min_width, min_height)
            .adjust_on_max_constraints(max_width, max_height);

        node.borrow_mut().rect.set_size(new_size);

        self
    }

    fn max_height(self, val: f32) -> Self {
        let node = self.node();
        node.borrow_mut().max_height = Some(val);

        let state = node.borrow();
        let min_width = state.min_width;
        let min_height = state.min_height;
        let max_width = state.max_width;
        let max_height = state.max_height;
        let current_size = state.rect.size();

        drop(state);

        let new_size = current_size
            .adjust_on_min_constraints(min_width, min_height)
            .adjust_on_max_constraints(max_width, max_height);

        node.borrow_mut().rect.set_size(new_size);

        self
    }

    fn align_h(self, align_h: AlignH) -> Self {
        self.node().borrow_mut().align_h = align_h;
        self
    }

    fn align_v(self, align_v: AlignV) -> Self {
        self.node().borrow_mut().align_v = align_v;
        self
    }
}

impl Widget for Box<dyn Widget> {
    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        self.as_mut().children_mut()
    }
}

impl Widget for Box<&mut dyn Widget> {
    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        self.as_mut().children_mut()
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

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

    #[allow(unused)]
    pub(crate) fn get(&self, event: WidgetEvent) -> Option<&Box<dyn FnMut()>> {
        self.0[event as usize].as_ref()
    }

    pub(crate) fn get_mut(&mut self, event: WidgetEvent) -> Option<&mut Box<dyn FnMut()>> {
        self.0[event as usize].as_mut()
    }
}

// -------------------------------------

pub(crate) struct WindowWidget {
    id: WidgetId,
    node: ViewNode,
    children: Vec<Box<dyn Widget>>,
}

impl WindowWidget {
    pub(crate) fn new(rect: Rect) -> Self {
        Self {
            id: WidgetId::new(),
            node: ViewNode::window(rect),
            children: Vec::new(),
        }
    }
}

impl Widget for WindowWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
    }
}

// -------------------------------------

pub struct CircleWidget {
    id: WidgetId,
    node: ViewNode,
}

impl CircleWidget {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            node: ViewNode::new()
                .with_name("Circle")
                .with_stroke_width(5.)
                .with_shape(Shape::Circle)
                .with_size((100., 100.)),
        }
    }
}

impl Widget for CircleWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
