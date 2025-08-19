use color::Rgba;
use image_data::{ImageData, ImageRef};

use crate::fraction::Fraction;

pub(crate) mod color;
pub(crate) mod image_data;

pub enum Paint {
    Color(Rgba),
    Image(ImageData),
    // TODO: Gradient,
}

pub enum PaintRef<'a> {
    Color(&'a Rgba),
    Image(ImageRef),
}

impl Paint {
    pub fn from_color(color: Rgba) -> Self {
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

    pub fn aspect_ratio(&self) -> Option<Fraction> {
        match self {
            Paint::Color(_) => None,
            Paint::Image(image_data) => Some(image_data.aspect_ratio()),
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

impl From<Rgba> for Paint {
    fn from(rgba: Rgba) -> Self {
        Self::Color(rgba)
    }
}

impl From<ImageData> for Paint {
    fn from(img: ImageData) -> Self {
        Self::Image(img)
    }
}
