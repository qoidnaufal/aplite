use aplite_types::{CornerRadius, Rgba, Size};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub(crate) size: Size,
    pub(crate) background: u32,
    pub(crate) border: u32,
    pub(crate) corners: u32,
    pub(crate) shape: Shape,
    pub(crate) border_width: f32,
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
    pub fn new(size: Size) -> Self {
        Self {
            size,
            background: Rgba::RED.into(),
            border: Rgba::WHITE.into(),
            corners: CornerRadius::splat(25).pack_u32(),
            shape: Shape::RoundedRect,
            border_width: 0.0,
        }
    }

    pub fn with_corner_radius(mut self, corner_radius: &CornerRadius) -> Self {
        self.corners = corner_radius.pack_u32();
        self
    }

    pub fn with_border_width(mut self, val: f32) -> Self {
        self.border_width = val;
        self
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
}
