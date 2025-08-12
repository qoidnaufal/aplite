use std::cell::RefCell;

use aplite_renderer::{Shape, Renderer};
use aplite_types::{Rgba, CornerRadius, Size};
use aplite_storage::U64Map;

use crate::state::AspectRatio;
use crate::context::layout::*;
use crate::view::{
    IntoView,
    ViewId,
    ViewNode,
    VIEW_STORAGE,
};

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
    fn node(&self) -> ViewNode;

    fn id(&self) -> ViewId {
        self.node().0
    }

    fn render(&self, renderer: &mut Renderer) {
        let scene = renderer.scene();
        let size = scene.size();

        let state = self.node().upgrade().unwrap();
        let state = state.borrow();

        let transform = state.get_transform(size);
        let rotation = state.rotation;
        let background_paint = state.background_paint.as_paint_ref();
        let border_paint = state.border_paint.as_paint_ref();
        let border_width = if state.border_width == 0.0 {
            5.0 / size.width
        } else {
            state.border_width / size.width
        };
        let shape = state.shape;

        scene.draw(
            transform,
            rotation,
            background_paint,
            border_paint,
            border_width,
            shape
        );
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn child(self, child: impl IntoView) -> Self {
        VIEW_STORAGE.with(|s| s.append_child(&self.id(), child));
        self
    }

    fn and(self, sibling: impl IntoView) -> Self {
        VIEW_STORAGE.with(|s| s.add_sibling(&self.id(), sibling));
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
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().image_aspect_ratio = val;
        }
        self
    }

    fn color(self, color: Rgba<u8>) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().background_paint = color.into();
        }
        self
    }

    fn border_color(self, color: Rgba<u8>) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().border_paint = color.into();
        }
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
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().border_width = val;
        }
        self
    }

    fn corners(self, corners: CornerRadius) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().corner_radius = corners;
        }
        self
    }

    fn shape(self, shape: Shape) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().shape = shape;
        }
        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().rect.set_size(size.into());
        }
        self
    }

    fn dragable(self) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().dragable = true;
        }
        VIEW_STORAGE.with(|s| {
            let mut hoverable = s.hoverable.borrow_mut();
            if !hoverable.contains(&self.id()) {
                hoverable.push(self.id());
            }
        });
        self
    }

    fn spacing(self, val: f32) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().spacing = val;
        }
        self
    }

    fn padding(self, padding: Padding) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().padding = padding;
        }
        self
    }

    fn min_width(self, val: f32) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().min_width = Some(val);
        }
        self
    }

    fn min_height(self, val: f32) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().min_height = Some(val);
        }
        self
    }

    fn max_width(self, val: f32) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().max_width = Some(val);
        }
        self
    }

    fn max_height(self, val: f32) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().max_height = Some(val);
        }
        self
    }

    fn align_h(self, align_h: AlignH) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().align_h = align_h;
        }
        self
    }

    fn align_v(self, align_v: AlignV) -> Self {
        if let Some(state) = self.node().upgrade() {
            state.borrow_mut().align_v = align_v;
        }
        self
    }
}

// -------------------------------------

thread_local! {
    pub(crate) static CALLBACKS: RefCell<U64Map<ViewId, CallbackStore>>
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
    node: ViewNode,
}

impl WindowWidget {
    pub(crate) fn new(size: Size) -> Self {
        Self {
            node: ViewNode::window(size)
        }
    }
}

impl Widget for WindowWidget {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}

// -------------------------------------

pub struct CircleWidget {
    node: ViewNode,
}

impl CircleWidget {
    pub fn new() -> Self {
        let node = ViewNode::new()
            .with_name("Circle")
            .with_stroke_width(5.)
            .with_shape(Shape::Circle)
            .with_size((100., 100.));

        Self {
            node,
        }
    }
}

impl Widget for CircleWidget {
    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}

// -------------------------------------

#[cfg(test)]
mod alt {
    use crate::state::WidgetState;
    use aplite_types::Rect;

    struct Button {
        state: WidgetState,
        children: Vec<Box<dyn Widget>>,
    }

    impl Button {
        fn new(name: &'static str) -> Self {
            Self {
                state: WidgetState::new().with_name(name),
                children: Vec::new(),
            }
        }
    }

    trait Widget {
        fn state_ref(&self) -> &WidgetState;
        fn state_mut(&mut self) -> &mut WidgetState;
        fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>>;
        fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>>;

        fn render(&self, renderer: &mut Vec<String>) {
            renderer.push(self.state_ref().name.to_string());

            if let Some(children) = self.children_ref() {
                children.iter()
                    .for_each(|w| w.render(renderer));
            }
        }

        fn layout(&mut self, parent_bound: Rect) {
            let offset = (parent_bound.x, parent_bound.y);
            self.state_mut().set_position(offset.into());
            let bound = self.state_ref().rect;
            if let Some(children) = self.children_mut() {
                children
                    .iter_mut()
                    .for_each(|w| w.layout(bound));
            }
        }
    }

    trait WidgetExt: Widget + Sized {
        fn child(mut self, child: impl Widget + 'static) -> Self {
            if let Some(children) = self.children_mut() {
                children.push(Box::new(child));
            }
            self
        }
    }

    impl<T: Widget + Sized> WidgetExt for T {}

    trait IntoView: Widget {
        fn into_view(self) -> View;
    }

    impl Widget for Button {
        fn state_ref(&self) -> &WidgetState {
            &self.state
        }

        fn state_mut(&mut self) -> &mut WidgetState {
            &mut self.state
        }

        fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
            Some(&self.children)
        }

        fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
            Some(&mut self.children)
        }
    }

    struct View {
        widget: Box<dyn Widget>,
    }

    impl View {
    }

    impl<T: Widget + 'static> IntoView for T {
        fn into_view(self) -> View {
            View {
                widget: Box::new(self),
            }
        }
    }

    impl Widget for View {
        fn state_ref(&self) -> &WidgetState {
            self.widget.as_ref().state_ref()
        }

        fn state_mut(&mut self) -> &mut WidgetState {
            self.widget.as_mut().state_mut()
        }

        fn children_ref(&self) -> Option<&Vec<Box<dyn Widget>>> {
            self.widget.as_ref().children_ref()
        }

        fn children_mut(&mut self) -> Option<&mut Vec<Box<dyn Widget>>> {
            self.widget.as_mut().children_mut()
        }
    }

    fn root() -> impl IntoView {
        Button::new("root")
            .child({
                Button::new("one")
                    .child(Button::new("four"))
                    .child(Button::new("five"))
            })
            .child(Button::new("two"))
            .child(Button::new("three"))
    }

    #[test]
    fn alt_test() {
        let mut renderer: Vec<String> = vec![];
        let screen = Rect::new(0.0, 0.0, 100.0, 100.0);

        let mut root = root().into_view();

        root.render(&mut renderer);
        root.layout(screen);

        eprintln!("{renderer:#?}");

        renderer.clear();
    }
}
