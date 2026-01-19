use color::Color;
use image_data::{ImageData, ImageRef};

use crate::fraction::Fraction;

pub(crate) mod color;
pub(crate) mod image_data;

#[derive(Clone)]
pub enum Paint {
    Color(Color),
    Image(ImageData),
    // TODO: Gradient,
}

pub enum PaintRef<'a> {
    Color(&'a Color),
    Image(ImageRef),
}

impl Paint {
    pub fn from_color(color: Color) -> Self {
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

impl PartialEq for Paint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Paint::Color(rgba), Paint::Color(rgba2)) => rgba == rgba2,
            (Paint::Image(image_data), Paint::Image(image_data2)) => image_data.eq(image_data2),
            _ => false
        }
    }
}

impl From<Color> for Paint {
    fn from(rgba: Color) -> Self {
        Self::Color(rgba)
    }
}

impl From<ImageData> for Paint {
    fn from(img: ImageData) -> Self {
        Self::Image(img)
    }
}

impl<'a> From<&'a Color> for PaintRef<'a> {
    fn from(rgba: &'a Color) -> Self {
        Self::Color(rgba)
    }
}

impl<'a> From<&'a ImageData> for PaintRef<'a> {
    fn from(img: &'a ImageData) -> Self {
        Self::Image(img.downgrade())
    }
}

impl<'a> From<ImageRef> for PaintRef<'a> {
    fn from(img: ImageRef) -> Self {
        Self::Image(img)
    }
}
