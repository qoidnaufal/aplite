 use shared::{Fraction, GpuPrimitive, Size, Rgba};

use crate::renderer::util::TextureDataSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixel<T: GpuPrimitive> {
    dimensions: Size<u32>,
    data: Vec<T>,
}

impl<T: GpuPrimitive> Pixel<T> {
    pub fn new(dimensions: impl Into<Size<u32>>, data: &[T]) -> Self {
        Self {
            dimensions: dimensions.into(),
            data: data.to_vec(),
        }
    }

    pub fn aspect_ratio(&self) -> Fraction<u32> {
        self.dimensions.aspect_ratio()
    }

    pub fn dimensions(&self) -> Size<u32> {
        self.dimensions
    }
}

impl<T: GpuPrimitive> std::ops::Deref for Pixel<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl From<Rgba<u8>> for Pixel<u8> {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new((1, 1), &rgba.to_slice())
    }
}

impl TextureDataSource for Pixel<u8> {
    fn data(&self) -> &[u8] { self }

    fn dimensions(&self) -> Size<u32> { self.dimensions }
}
