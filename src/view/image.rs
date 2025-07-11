use aplite_renderer::Shape;
use aplite_renderer::ImageData;

use crate::context::widget_state::WidgetState;
use crate::prelude::AspectRatio;

use super::ViewNode;
use super::ViewId;
use super::Widget;
use super::VIEW_STORAGE;

pub fn image<F: Fn() -> ImageData + 'static>(image_fn: F) -> Image {
    Image::new(image_fn)
}

pub struct Image {
    id: ViewId,
    node: ViewNode,
    state: WidgetState,
}

impl Image {
    pub fn new<F: Fn() -> ImageData + 'static>(f: F) -> Self {
        let id = VIEW_STORAGE.with(|s| {
            let id = s.create_entity();
            s.image_fn.borrow_mut().insert(id, Box::new(f));
            id
        });
        let node = ViewNode::new()
            .with_shape(Shape::Rect);
        let state = WidgetState::new()
            .with_name("Image")
            .with_size((100, 100));

        Self {
            id,
            state,
            node,
        }
    }

    // pub fn append_child(self, child: impl IntoView) -> Self {
    //     VIEW_STORAGE.with(|s| s.append_child(&self.id, child));
    //     self
    // }

    // pub fn and(self, sibling: impl IntoView) -> Self {
    //     VIEW_STORAGE.with(|s| s.add_sibling(&self.id, sibling));
    //     self
    // }

    pub fn with_aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
        self.state.set_image_aspect_ratio(aspect_ratio);
        self
    }

    pub fn state(mut self, mut f: impl FnMut(&mut WidgetState) + 'static) -> Self {
        f(&mut self.state);
        self
    }
}

impl Widget for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn widget_state(&self) -> WidgetState {
        self.state
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
