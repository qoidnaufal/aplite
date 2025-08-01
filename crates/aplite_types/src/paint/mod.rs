pub(crate) mod color;
pub(crate) mod image_data;

use color::Rgba;
use image_data::{ImageData, ImageRef};

pub enum Paint {
    Color(Rgba<u8>),
    Image(ImageData),
    // TODO: Gradient,
}

pub enum PaintRef<'a> {
    Color(&'a Rgba<u8>),
    Image(ImageRef),
}

impl Paint {
    pub fn from_color(color: Rgba<u8>) -> Self {
        Self::Color(color)
    }

    pub fn from_image(image: ImageData) -> Self {
        Self::Image(image)
    }

    pub fn as_paint_ref(&self) -> PaintRef<'_> {
        match self {
            Paint::Color(rgba) => PaintRef::Color(rgba),
            Paint::Image(image_data) => PaintRef::Image(image_data.downgrade()),
        }
    }
}

impl Clone for Paint {
    fn clone(&self) -> Self {
        match self {
            Paint::Color(rgba) => Paint::Color(*rgba),
            Paint::Image(image_data) => Paint::Image(image_data.clone()),
        }
    }
}

impl PartialEq for Paint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Paint::Color(rgba), Paint::Color(rgba2)) => rgba == rgba2,
            (Paint::Image(image_data), Paint::Image(image_data2)) => image_data.eq(image_data2),
            _ => false
        }
    }
}

impl From<Rgba<u8>> for Paint {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::Color(rgba)
    }
}

impl From<ImageData> for Paint {
    fn from(img: ImageData) -> Self {
        Self::Image(img)
    }
}
