mod atlas;
mod image_data;
mod texture_data;

use aplite_types::Rect;

pub(crate) use atlas::Atlas;
pub(crate) use texture_data::TextureData;
pub use image_data::{ImageData, image_reader};

#[derive(Debug, Clone, Copy)]
pub enum TextureInfo {
    ImageId(i32),
    AtlasId {
        id: i32,
        uv: Rect<f32>,
    }
}

impl TextureInfo {
    pub fn get_uv(&self) -> Option<Rect<f32>> {
        match self {
            TextureInfo::ImageId(_) => None,
            TextureInfo::AtlasId { uv, .. } => Some(*uv),
        }
    }

    pub fn get_image_id(&self) -> Option<i32> {
        match self {
            TextureInfo::ImageId(id) => Some(*id),
            TextureInfo::AtlasId { .. } => None,
        }
    }

    pub fn get_atlas_id(&self) -> Option<i32> {
        match self {
            TextureInfo::ImageId(_) => None,
            TextureInfo::AtlasId { id, .. } => Some(*id),
        }
    }
}
