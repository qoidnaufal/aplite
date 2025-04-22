use util::tan;
use crate::cursor::Cursor;
use crate::layout::Attributes;
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

    pub fn fill_color(&self) -> Rgba<u8> {
        self.fill_color.into()
    }

    pub fn set_fill_color<F: FnOnce(&mut Rgba<u8>)>(&mut self, f: F) {
        let mut rgba = self.fill_color.into();
        f(&mut rgba);
        self.fill_color = rgba.into();
    }

    pub(crate) fn revert_color(&mut self, cached_color: Rgba<u8>) {
        self.fill_color = cached_color.into();
    }


    pub(crate) fn is_hovered(&self, cursor: &Cursor, attr: &Attributes) -> bool {
        // let rotate = Matrix2x2::rotate(self.rotate);
        // let pos: Vector2<f32> = attr.pos.into();
        // let p = rotate * pos;
        let x = attr.pos.x as f32;
        let y = attr.pos.y as f32;

        let x_cursor = cursor.hover.pos.x;
        let y_cursor = cursor.hover.pos.y;

        let width = attr.size.width as f32 / 2.0;
        let height = attr.size.height as f32 / 2.0;

        let angled = if Shape::from(self.shape).is_triangle() {
            let c_tangen = tan(x_cursor - x, y_cursor - y + height);
            let t_tangen = tan(width / 2.0, height);
            (t_tangen - c_tangen).is_sign_negative()
        } else { true };

        (y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
            && angled
    }
}

