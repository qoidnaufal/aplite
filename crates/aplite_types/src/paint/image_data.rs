use std::sync::{Arc, Weak};
use crate::{Fraction, Rgba, Size};

pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub bytes: Arc<Vec<u8>>,
}

impl ImageData {
    pub fn new((width, height): (u32, u32), data: &[u8]) -> Self {
        Self { width, height, bytes: Arc::new(data.to_vec()) }
    }

    pub fn downgrade(&self) -> ImageRef {
        ImageRef {
            width: self.width,
            height: self.height,
            bytes: Arc::downgrade(&self.bytes),
        }
    }

    pub fn aspect_ratio(&self) -> Fraction {
        Size::new(self.width as f32, self.height as f32).aspect_ratio()
    }
}

impl std::ops::Deref for ImageData {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.bytes.as_slice()
    }
}

impl From<Rgba<u8>> for ImageData {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new((1, 1), &rgba.as_slice())
    }
}

impl Clone for ImageData {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            bytes: Arc::clone(&self.bytes),
        }
    }
}

impl std::hash::Hash for ImageData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.bytes) as usize);
    }
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.bytes, &other.bytes)
    }
}

impl PartialEq<ImageRef> for ImageData {
    fn eq(&self, other: &ImageRef) -> bool {
        other.bytes
            .upgrade()
            .is_some_and(|bytes| {
                Arc::ptr_eq(&bytes, &self.bytes)
            })
    }
}

impl Eq for ImageData {}

pub struct ImageRef {
    pub width: u32,
    pub height: u32,
    pub bytes: Weak<Vec<u8>>,
}

impl ImageRef {
    pub fn upgrade(&self) -> Option<ImageData> {
        self.bytes
            .upgrade()
            .map(|bytes| {
                ImageData {
                    width: self.width,
                    height: self.height,
                    bytes,
                }
            })
    }
}

impl PartialEq<ImageData> for ImageRef {
    fn eq(&self, other: &ImageData) -> bool {
        self
            .bytes
            .upgrade()
            .is_some_and(|bytes| {
                Arc::ptr_eq(&bytes, &other.bytes)
            })
    }
}

impl PartialEq for ImageRef {
    fn eq(&self, other: &Self) -> bool {
        if let Some(byte1) = self.bytes.upgrade()
        && let Some(byte2) = other.bytes.upgrade() {
            Arc::ptr_eq(&byte1, &byte2)
        } else {
            false
        }
    }
}

impl Eq for ImageRef {}

impl std::hash::Hash for ImageRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(Weak::as_ptr(&self.bytes) as usize);
    }
}
