use aplite_renderer::Shape;
use aplite_renderer::ImageData;

use crate::context::widget_state::WidgetState;

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

    pub fn state(mut self, f: impl Fn(&mut WidgetState)) -> Self {
        f(&mut self.state);
        self
    }
}

impl Widget for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
