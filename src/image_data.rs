 use std::{fs::File, io::Read, path::PathBuf};

use image::GenericImageView;
use shared::{Fraction, Size, Rgba};

use crate::renderer::util::TextureDataSource;

pub fn image_reader<P: Into<PathBuf>>(path: P) -> ImageData {
    let mut file = File::open(path.into()).unwrap();
    let mut buf = Vec::new();
    let len = file.read_to_end(&mut buf).unwrap();
    let image = image::load_from_memory(&buf[..len]).unwrap();

    ImageData::new(image.dimensions(), &image.to_rgba8())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    dimensions: Size<u32>,
    data: Vec<u8>,
}

impl ImageData {
    pub fn new(dimensions: impl Into<Size<u32>>, data: &[u8]) -> Self {
        Self {
            dimensions: dimensions.into(),
            data: data.to_vec(),
        }
    }

    pub(crate) fn aspect_ratio(&self) -> Fraction<u32> {
        self.dimensions.aspect_ratio()
    }

    pub(crate) fn dimensions(&self) -> Size<u32> {
        self.dimensions
    }
}

impl std::ops::Deref for ImageData {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl From<Rgba<u8>> for ImageData {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new((1, 1), &rgba.to_slice())
    }
}

impl TextureDataSource for ImageData {
    fn data(&self) -> &[u8] { self }

    fn dimensions(&self) -> Size<u32> { self.dimensions }
}
