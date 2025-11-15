use aplite_types::{CornerRadius, Size};
use aplite_types::theme::basic;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub(crate) size: Size,
    pub(crate) background: u32,
    pub(crate) border: u32,
    pub(crate) corners: u32,
    pub(crate) shape: u32,
    pub(crate) border_width: f32,
}

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
            background: basic::RED.into(),
            border: basic::WHITE.into(),
            corners: 0,
            shape: 1,
            border_width: 0.0,
        }
    }

    pub(crate) fn with_corner_radius(mut self, corner_radius: &CornerRadius) -> Self {
        self.corners = corner_radius.pack_u32();
        self
    }

    pub(crate) fn with_border_width(mut self, val: f32) -> Self {
        self.border_width = val;
        self
    }

    pub(crate) fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape as u32;
        self
    }
}
