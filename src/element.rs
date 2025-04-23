use crate::renderer::Indices;
use crate::style::{Corners, Shape, Style};
use crate::color::Rgba;


#[repr(C)]
#[derive(Debug, Clone)]
pub struct Element {
    fill_color: Rgba<f32>,
    stroke_color: Rgba<f32>,
    corners: Corners,
    shape: u32,
    rotate: f32,
    stroke_width: f32,
    pub texture_id: i32,
    pub transform_id: u32,
}

impl Element {
    pub fn filled(style: &Style) -> Self {
        Self {
            fill_color: style.fill_color().into(),
            stroke_color: style.stroke_color().into(),
            shape: style.shape() as u32,
            corners: style.corners(),
            rotate: style.rotation(),
            stroke_width: style.stroke_width(),
            texture_id: -1,
            transform_id: 0,
        }
    }

    pub fn textured(style: &Style) -> Self {
        Self {
            fill_color: style.fill_color().into(),
            stroke_color: style.stroke_color().into(),
            shape: style.shape() as u32,
            corners: style.corners(),
            rotate: style.rotation(),
            stroke_width: style.stroke_width(),
            texture_id: 0,
            transform_id: 0,
        }
    }

    pub(crate) fn indices<'a>(&self) -> Indices<'a> {
        Indices::from(Shape::from(self.shape))
    }

    pub fn color(&self) -> Rgba<u8> {
        self.fill_color.into()
    }

    pub fn update_color<F: FnOnce(&mut Rgba<u8>)>(&mut self, f: F) {
        let mut rgba = self.fill_color.into();
        f(&mut rgba);
        self.fill_color = rgba.into();
    }

    pub(crate) fn set_color(&mut self, color: Rgba<u8>) {
        self.fill_color = color.into();
    }
}

