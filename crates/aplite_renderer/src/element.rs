use aplite_types::{CornerRadius, Rgba};

use crate::texture::AtlasId;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub(crate) fill_color: Rgba<f32>,
    pub(crate) stroke_color: Rgba<f32>,
    pub(crate) corners: CornerRadius,
    pub(crate) shape: Shape,
    pub(crate) rotation: f32,
    pub(crate) stroke_width: f32,
    pub(crate) atlas_id: AtlasId,
    pub(crate) transform_id: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle = 0,
    Rect = 1,
    RoundedRect = 2,
    Triangle = 3,
}

impl Element {
    pub const fn new() -> Self {
        Self {
            fill_color: Rgba::new(1., 0., 0., 1.),
            stroke_color: Rgba::new(1., 1., 1., 1.),
            corners: CornerRadius::splat(25.),
            shape: Shape::RoundedRect,
            rotation: 0.0,
            stroke_width: 0.0,
            atlas_id: AtlasId::new(-1),
            transform_id: 0,
        }
    }

    pub(crate) fn atlas_id(&self) -> AtlasId {
        self.atlas_id
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn with_fill_color(mut self, color: Rgba<u8>) -> Self {
        self.fill_color = color.into();
        self
    }

    pub fn with_stroke_color(mut self, color: Rgba<u8>) -> Self {
        self.stroke_color = color.into();
        self
    }

    pub fn with_corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corners = corner_radius;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    // pub(crate) fn with_transform_id(mut self, id: u32) -> Self {
    //     self.transform_id = id;
    //     self
    // }

    // pub(crate) fn with_atlas_id(mut self, id: i32) -> Self {
    //     self.atlas_id = id;
    //     self
    // }

    // pub(crate) fn with_image_id(mut self, id: i32) -> Self {
    //     self.image_id = id;
    //     self
    // }

    pub fn set_fill_color(&mut self, color: Rgba<u8>) {
        self.fill_color = color.into();
    }

    pub fn set_stroke_color(&mut self, color: Rgba<u8>) {
        self.stroke_color = color.into();
    }

    pub fn set_stroke_width(&mut self, val: u32) {
        self.stroke_width = val as _;
    }

    pub fn set_rotation(&mut self, val: f32) {
        self.rotation = val;
    }

    pub fn set_corner_radius(&mut self, val: CornerRadius) {
        self.corners = val;
    }

    pub fn set_shape(&mut self, shape: Shape) {
        self.shape = shape;
    }

    pub fn set_transform_id(&mut self, val: u32) {
        self.transform_id = val;
    }

    pub fn set_atlas_id(&mut self, id: AtlasId) {
        self.atlas_id = id;
    }

    pub fn fill_color(&self) -> Rgba<u8> {
        self.fill_color.u8()
    }
}
