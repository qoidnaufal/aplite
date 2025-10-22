use std::path::Path;

use aplite_renderer::{Shape, Scene, DrawArgs};
use aplite_types::ImageData;

use crate::state::WidgetState;
use super::{Widget, WidgetId};

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
    state: WidgetState,
}

impl Image {
    pub fn new<F: Fn() -> ImageData + 'static>(image_fn: F) -> Self {
        let state = WidgetState::default()
            .with_size(100.0, 100.0)
            .with_background_paint(image_fn())
            .with_shape(Shape::Rect);

        Self {
            state,
        }
    }
}

impl Widget for Image {
    fn state_ref(&self) -> &WidgetState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn draw(&self, scene: &mut Scene) {
        if self.state.flag.visible {
            scene.draw_rect(
                &self.state.rect,
                &self.state.transform,
                &self.state.background.paint.as_paint_ref(),
                &self.state.border.paint.as_paint_ref(),
                &self.state.border.width,
            );
        }
    }
}
