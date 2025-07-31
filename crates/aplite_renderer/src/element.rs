use aplite_types::{CornerRadius, Rgba};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub(crate) background: Rgba<f32>,
    pub(crate) border: Rgba<f32>,
    pub(crate) corners: CornerRadius,
    pub(crate) shape: Shape,
    pub(crate) border_width: f32,
    pub(crate) atlas_id: i32,
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
            background: Rgba::new(1., 0., 0., 1.),
            border: Rgba::new(1., 1., 1., 1.),
            corners: CornerRadius::splat(25.),
            shape: Shape::RoundedRect,
            border_width: 0.0,
            atlas_id: -1,
            transform_id: 0,
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

    pub fn with_corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corners = corner_radius;
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

    pub(crate) fn with_transform_id(mut self, id: u32) -> Self {
        self.transform_id = id;
        self
    }

    // pub(crate) fn with_atlas_id(mut self, id: i32) -> Self {
    //     self.atlas_id = id;
    //     self
    // }

    // pub fn set_background(&mut self, color: Rgba<u8>) {
    //     self.background = color.into();
    // }

    // pub fn set_stroke_color(&mut self, color: Rgba<u8>) {
    //     self.border = color.into();
    // }

    // pub fn set_stroke_width(&mut self, val: u32) {
    //     self.border_width = val as _;
    // }

    // pub fn set_rotation(&mut self, val: f32) {
    //     self.rotation = val;
    // }

    // pub fn set_corner_radius(&mut self, val: CornerRadius) {
    //     self.corners = val;
    // }

    // pub fn set_shape(&mut self, shape: Shape) {
    //     self.shape = shape;
    // }

    pub fn set_transform_id(&mut self, val: u32) {
        self.transform_id = val;
    }

    pub fn set_atlas_id(&mut self, id: i32) {
        self.atlas_id = id;
    }

    pub fn fill_color(&self) -> Rgba<u8> {
        self.background.u8()
    }
}
