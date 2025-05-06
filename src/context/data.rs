use util::{Size, Vector2};

use crate::color::Rgba;
use crate::renderer::Corners;
use crate::renderer::Shape;

#[derive(Default)]
pub(crate) struct LayoutData {
    pos: Vector2<u32>,
    size: Size<u32>,
}

impl LayoutData {
    pub(crate) fn new(pos: Vector2<u32>, size: Size<u32>) -> Self {
        Self { pos, size }
    }
}

pub(crate) struct StyleData {
    hover_color: Option<Rgba<u8>>,
    click_color: Option<Rgba<u8>>,
    fill_color: Rgba<u8>,
    stroke_color: Rgba<u8>,
    shape: Shape,
    corners: Corners,
    rotation: f32,
    stroke_width: f32,
    texture_id: i32,
}

pub(crate) struct Data {
    layout: Vec<LayoutData>,
    style: Vec<StyleData>,
}
