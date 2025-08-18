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

/// main building block to create a renderable component
pub trait Widget {
    fn node_ref(&self) -> NodeRef;

    fn id(&self) -> WidgetId {
        self.node_ref().id()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        None
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        None
    }

    fn draw(&self, scene: &mut Scene) {
        let node = self.node_ref().upgrade();

        if !node.borrow().hide {
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

            if let Some(children) = self.children_ref() {
                children
                    .iter()
                    .for_each(|child| child.draw(scene));
            }
        }
    }

    fn layout(&self, cx: &mut LayoutCx) -> bool {
        let node = self.node_ref().upgrade();
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
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .image_aspect_ratio = val;

        self
    }

    fn color(self, color: Rgba<u8>) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .background_paint = color.into();

        self
    }

    fn border_color(self, color: Rgba<u8>) -> Self {
        self.node_ref()
            .upgrade()
            .borrow_mut()
            .border_paint = color.into();

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
        node.dragable = true;
        node.hoverable = true;

        self
    }

    fn spacing(self, val: f32) -> Self {
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

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        self.as_mut().children_mut()
    }
}

impl Widget for Box<&mut dyn Widget> {
    fn node_ref(&self) -> NodeRef {
        self.as_ref().node_ref()
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        self.as_mut().children_mut()
    }
}

impl Widget for *const dyn Widget {
    fn node_ref(&self) -> NodeRef {
        unsafe {
            self.as_ref().unwrap().node_ref()
        }
    }

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        unsafe {
            self.as_ref().and_then(|w| w.children_ref())
        }
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
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

    // #[allow(unused)]
    // pub(crate) fn get(&self, event: WidgetEvent) -> Option<&Box<dyn FnMut()>> {
    //     self.0[event as usize].as_ref()
    // }

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

    fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
        Some(&self.children)
    }

    fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
        Some(&mut self.children)
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
