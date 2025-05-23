use aplite_types::{Fraction, Rect, Rgba, Size};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    pub(crate) rect: Rect<u32>,
    pub(crate) data: Vec<u8>,
}

impl ImageData {
    pub fn new(size: impl Into<Size<u32>>, data: &[u8]) -> Self {
        let size: Size<u32> = size.into();
        Self {
            rect: Rect::new((0, 0), (size.width(), size.height())),
            data: data.to_vec(),
        }
    }

    pub const fn rect(&self) -> Rect<u32> { self.rect }

    pub fn aspect_ratio(&self) -> Fraction<u32> {
        self.rect.size().aspect_ratio()
    }

    pub(crate) const fn size(&self) -> Size<u32> { self.rect.size() }

    pub(crate) const fn width(&self) -> u32 { self.rect.width() }

    pub(crate) const fn height(&self) -> u32 { self.rect.height() }

    // pub(crate) const fn x(&self) -> u32 { self.rect.x() }

    // pub(crate) const fn y(&self) -> u32 { self.rect.y() }
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
