use aplite_types::{CornerRadius, Rgba, Size};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub(crate) background: Rgba<f32>,
    pub(crate) border: Rgba<f32>,
    pub(crate) corners: CornerRadius,
    pub(crate) size: Size,
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
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            background: Rgba::new(1., 0., 0., 1.),
            border: Rgba::new(1., 1., 1., 1.),
            corners: CornerRadius::splat(25.),
            shape: Shape::RoundedRect,
            border_width: 0.0,
        }
    }

    pub fn with_background(mut self, color: Rgba<f32>) -> Self {
        self.background = color;
        self
    }

    pub fn with_border(mut self, color: Rgba<f32>) -> Self {
        self.border = color;
        self
    }

    pub fn with_corner_radius(mut self, corner_radius: &CornerRadius) -> Self {
        self.corners.tl = corner_radius.tl;
        self.corners.bl = corner_radius.bl;
        self.corners.br = corner_radius.br;
        self.corners.tr = corner_radius.tr;
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

    pub fn fill_color(&self) -> Rgba<u8> {
        self.background.u8()
    }
}
