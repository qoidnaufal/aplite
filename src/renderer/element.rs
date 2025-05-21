use shared::{Rgba, Size};

use super::util::RenderComponentSource;
// use super::{Indices, RenderComponentSource};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle = 0,
    Rect = 1,
    RoundedRect = 2,
    Triangle = 3,
}

// impl Shape {
//     pub(crate) fn is_triangle(&self) -> bool { matches!(self, Self::Triangle) }

//     pub(crate) fn is_rounded_rect(&self) -> bool { matches!(self, Self::RoundedRect) }
// }

#[repr(C, align(16))]
#[derive(Default, Debug, Clone, Copy)]
pub struct CornerRadius {
    tl: f32,
    bl: f32,
    br: f32,
    tr: f32,
}

impl From<u32> for CornerRadius {
    fn from(val: u32) -> Self {
        Self {
            tl: val as _,
            bl: val as _,
            br: val as _,
            tr: val as _,
        }
    }
}

impl CornerRadius {
    pub const fn new_homogen(r: u32) -> Self {
        Self {
            tl: r as _,
            bl: r as _,
            br: r as _,
            tr: r as _,
        }
    }

    pub fn set_each(&mut self, tl: u32, bl: u32, br: u32, tr: u32) {
        self.tl = tl as _;
        self.bl = bl as _;
        self.br = br as _;
        self.tr = tr as _;
    }

    pub fn set_all(&mut self, r: u32) {
        self.tl = r as _;
        self.bl = r as _;
        self.br = r as _;
        self.tr = r as _;
    }

    pub fn set_top_left(&mut self, r: u32) {
        self.tl = r as _;
    }

    pub fn set_bot_left(&mut self, r: u32) {
        self.bl = r as _;
    }

    pub fn set_bot_right(&mut self, r: u32) {
        self.br = r as _;
    }

    pub fn set_top_right(&mut self, r: u32) {
        self.tr = r as _;
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct Element {
    pub(crate) fill_color: Rgba<f32>,
    pub(crate) stroke_color: Rgba<f32>,
    pub(crate) corners: CornerRadius,
    pub(crate) size: Size<f32>,
    pub(crate) shape: Shape,
    pub(crate) rotation: f32,
    pub(crate) stroke_width: f32,
    pub(crate) texture_id: i32,
    pub(crate) transform_id: u32,
}

impl Element {
    pub(crate) fn new(rcs: &impl RenderComponentSource) -> Self {
        Self {
            fill_color: rcs.fill_color(),
            stroke_color: rcs.stroke_color(),
            corners: rcs.corners(),
            size: rcs.rect().size(),
            shape: rcs.shape(),
            rotation: rcs.rotation(),
            stroke_width: rcs.stroke_width(),
            texture_id: rcs.texture_id(),
            transform_id: 0,
        }
    }

    pub(crate) fn with_transform_id(mut self, id: u32) -> Self {
        self.transform_id = id;
        self
    }

    pub(crate) fn set_color(&mut self, color: Rgba<u8>) {
        self.fill_color = color.into();
    }

    pub(crate) fn set_size(&mut self, size: Size<u32>) {
        self.size = size.into();
    }
}

