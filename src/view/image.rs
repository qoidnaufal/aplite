use std::path::Path;

use aplite_renderer::Shape;
use aplite_types::ImageData;
use aplite_types::Paint;

use crate::widget_state::WidgetState;

use super::ViewNode;
use super::{ViewId, PaintId};
use super::Widget;
use super::VIEW_STORAGE;

pub fn image<F: Fn() -> ImageData + 'static>(image_fn: F) -> Image {
    Image::new(image_fn)
}

/// This function will resize the image to 500x500 by default to optimize gpu performance.
/// If you want to have your image bytes fully rendered, consider to use your own function
pub fn image_reader<P: AsRef<Path>>(path: P) -> ImageData {
    use image::imageops::FilterType;
    use image::{GenericImageView, ImageReader};

    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .resize_to_fill(500, 500, FilterType::Lanczos3);

    ImageData::new(img.dimensions(), &img.to_rgba8())
}

pub struct Image {
    id: ViewId,
    paint_id: PaintId,
    node: ViewNode,
}

impl Image {
    pub fn new<F: Fn() -> ImageData + 'static>(f: F) -> Self {
        let (id, paint_id) = VIEW_STORAGE.with(|s| {
            let state = WidgetState::new()
                .with_name("Image")
                .with_size((100, 100));
            let id = s.insert(state);
            let paint = Paint::from_image(f());
            let paint_id = s.add_paint(paint);
            (id, paint_id)
        });
        let node = ViewNode::new()
            .with_shape(Shape::Rect);

        Self {
            id,
            node,
            paint_id,
        }
    }

    pub fn state(self, f: impl Fn(&mut WidgetState)) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_data_mut(&self.id).unwrap();
            f(state);
        });
        self
    }
}

impl Widget for Image {
    fn id(&self) -> ViewId {
        self.id
    }

    fn paint_id(&self) -> PaintId {
        self.paint_id
    }

    fn node(&self) -> ViewNode {
        self.node.clone()
    }
}
