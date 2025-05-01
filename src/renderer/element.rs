use super::Indices;
use crate::properties::Shape;
use crate::color::Rgba;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
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
    pub fn new_homogen(r: f32) -> Self {
        r.into()
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
pub struct Element {
    fill_color: Rgba<f32>,
    stroke_color: Rgba<f32>,
    corners: Corners,
    shape: u32,
    rotation: f32,
    stroke_width: f32,
    pub texture_id: i32,
    pub transform_id: u32,
}

impl Element {
    pub(crate) fn new(
        fill_color: Rgba<f32>,
        stroke_color: Rgba<f32>,
        corners: Corners,
        shape: u32,
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

