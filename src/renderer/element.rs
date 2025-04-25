use super::Indices;
use crate::properties::{Corners, Shape, Properties};
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
    pub(crate) fn new(properties: &Properties) -> Self {
        Self {
            fill_color: properties.fill_color().into(),
            stroke_color: properties.stroke_color().into(),
            shape: properties.shape() as u32,
            corners: properties.corners(),
            rotate: properties.rotation(),
            stroke_width: properties.stroke_width(),
            texture_id: if properties.is_textured() { 0 } else { -1 },
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

