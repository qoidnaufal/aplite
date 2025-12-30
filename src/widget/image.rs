use std::path::Path;

use aplite_types::{ImageData, Length, Matrix3x2, PaintRef, Rect, Size, rgba};
use aplite_renderer::Scene;

use crate::context::Context;
use crate::layout::LayoutCx;
use crate::view::{ForEachView, IntoView};
use crate::widget::Widget;

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

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined(u8, u8),
    Source,
    Undefined,
}


pub struct Image {
    width: Length,
    height: Length,
    aspect_ratio: AspectRatio,
    data: ImageData,
}

impl Image {
    fn new<F: Fn() -> ImageData + 'static>(image_fn: F) -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            aspect_ratio: AspectRatio::Source,
            data: image_fn(),
        }
    }
}

impl Widget for Image {
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout_node_size(&self) -> Size {
        Size::new(self.data.width as _, self.data.height as _)
    }

    fn layout(&self, _: &mut LayoutCx<'_>) {
        let _ = self.aspect_ratio;
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        scene.draw(aplite_renderer::DrawArgs {
            rect: &Rect::from_size((self.data.width, self.data.height).into()),
            transform: &Matrix3x2::identity(),
            background_paint: &PaintRef::Image(self.data.downgrade()),
            border_paint: &PaintRef::Color(&rgba(0x00000000)),
            border_width: &0.,
            shape: &aplite_renderer::Shape::Rect,
            corner_radius: &aplite_types::CornerRadius::splat(0),
        });
    }
}

impl ForEachView for Image {}

impl IntoView for Image {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}
