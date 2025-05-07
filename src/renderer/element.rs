use shared::Rgba;
use super::{Indices, RenderComponentSource};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle = 0,
    Rect = 1,
    RoundedRect = 2,
    Triangle = 3,
}

impl Shape {
    pub(crate) fn is_triangle(&self) -> bool { matches!(self, Self::Triangle) }

    // pub(crate) fn is_rounded_rect(&self) -> bool { matches!(self, Self::RoundedRect) }
}

#[repr(C, align(16))]
#[derive(Default, Debug, Clone, Copy)]
pub struct Corners {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
}

impl From<f32> for Corners {
    fn from(val: f32) -> Self {
        Self {
            top_left: val,
            bot_left: val,
            bot_right: val,
            top_right: val,
        }
    }
}

impl Corners {
    pub const fn new_homogen(r: f32) -> Self {
        Self {
            top_left: r,
            bot_left: r,
            bot_right: r,
            top_right: r,
        }
    }

    pub fn set_all(&mut self, tl: f32, bl: f32, br: f32, tr: f32) {
        self.top_left = tl;
        self.bot_left = bl;
        self.bot_right = br;
        self.top_right = tr;
    }

    pub fn set_each(&mut self, r: f32) {
        self.top_left = r;
        self.bot_left = r;
        self.bot_right = r;
        self.top_right = r;
    }

    pub fn set_top_left(&mut self, r: f32) {
        self.top_left = r;
    }

    pub fn set_bot_left(&mut self, r: f32) {
        self.bot_left = r;
    }

    pub fn set_bot_right(&mut self, r: f32) {
        self.bot_right = r;
    }

    pub fn set_top_right(&mut self, r: f32) {
        self.top_right = r;
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct Element {
    fill_color: Rgba<f32>,
    stroke_color: Rgba<f32>,
    corners: Corners,
    shape: Shape,
    rotation: f32,
    stroke_width: f32,
    pub(crate) texture_id: i32,
    pub(crate) transform_id: u32,
}

impl Element {
    pub(crate) fn new(
        fill_color: Rgba<f32>,
        stroke_color: Rgba<f32>,
        corners: Corners,
        shape: Shape,
        rotation: f32,
        stroke_width: f32,
        texture_id: i32,
    ) -> Self {
        Self {
            fill_color,
            stroke_color,
            shape,
            corners,
            rotation,
            stroke_width,
            texture_id,
            transform_id: 0,
        }
    }

    // pub(crate) fn update(&mut self, rcs: impl RenderComponentSource) {
    //     self.fill_color =  rcs.fill_color();
    //     self.stroke_color =  rcs.stroke_color();
    //     self.corners =  rcs.corners();
    //     self.shape =  rcs.shape();
    //     self.rotation =  rcs.rotation();
    //     self.stroke_width =  rcs.stroke_width();
    // }

    pub(crate) fn indices<'a>(&self) -> Indices<'a> {
        Indices::from(Shape::from(self.shape))
    }

    // pub fn color(&self) -> Rgba<u8> {
    //     self.fill_color.into()
    // }

    // pub fn update_color<F: FnOnce(&mut Rgba<u8>)>(&mut self, f: F) {
    //     let mut rgba = self.fill_color.into();
    //     f(&mut rgba);
    //     self.fill_color = rgba.into();
    // }

    pub(crate) fn set_color(&mut self, color: Rgba<u8>) {
        self.fill_color = color.into();
    }
}

