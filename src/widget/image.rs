use std::path::Path;

use aplite_renderer::Scene;
use aplite_types::{ImageData, ImageRef, Length, Matrix3x2, PaintRef, Rect, rgb};

use crate::context::{BuildCx, LayoutCx, CursorCx};
use crate::layout::Axis;
use crate::widget::{Renderable, Widget};

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
    data: ImageData,
}

impl Image {
    fn new<F: Fn() -> ImageData + 'static>(image_fn: F) -> Self {
        Self {
            data: image_fn(),
        }
    }
}

impl Widget for Image {
    fn build(&self, cx: &mut BuildCx<'_>) {
        cx.register_element(ImageElement::new(&self.data));
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        let state = cx.get_element::<ImageElement>().unwrap();
        let bound = cx.bound;

        let width = match state.width {
            Length::Grow => bound.width,
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let height = match state.height {
            Length::Grow => bound.height,
            Length::Fixed(val) => val,
            Length::FitContent => 0.,
        };

        let layout_node = Rect::new(bound.x, bound.y, width, height);

        match cx.rules.axis {
            Axis::Horizontal => {
                cx.bound.x += width + cx.rules.spacing.0 as f32;
            },
            Axis::Vertical =>  {
                cx.bound.y += height + cx.rules.spacing.0 as f32;
            },
        }

        cx.set_node(layout_node);
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        let rect = cx.get_layout_node().unwrap();
        rect.contains(cx.hover_pos())
    }
}

pub struct ImageElement {
    pub width: Length,
    pub height: Length,
    pub aspect_ratio: AspectRatio,
    data: ImageRef,
}

impl std::fmt::Debug for ImageElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageElement")
            .finish_non_exhaustive()
    }
}

impl ImageElement {
    fn new(data: &ImageData) -> Self {
        Self {
            width: Length::Grow,
            height: Length::Grow,
            aspect_ratio: AspectRatio::Source,
            data: data.downgrade(),
        }
    }
}

impl Renderable for ImageElement {
    fn render(&self, rect: &Rect, scene: &mut Scene) {
        scene.draw_rect(
            rect,
            &Matrix3x2::identity(),
            &PaintRef::from(self.data.clone()),
            &PaintRef::from(&rgb(0x000000)),
            &0.
        );
    }
}
