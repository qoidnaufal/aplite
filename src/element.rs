use util::{tan, Matrix4x4, Size, Vector2};
use crate::context::Cursor;
use crate::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Rect,
    RoundedRect,
    Triangle,
}

impl Shape {
    fn is_triangle(&self) -> bool { matches!(self, Self::Triangle) }

    fn is_rounded_rect(&self) -> bool { matches!(self, Self::RoundedRect) }
}

impl From<u32> for Shape {
    fn from(num: u32) -> Self {
        match num {
            0 => Self::Circle,
            1 => Self::Rect,
            2 => Self::RoundedRect,
            3 => Self::Triangle,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Indices<'a>(&'a [u32]);

impl std::ops::Deref for Indices<'_> {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl From<Shape> for Indices<'_> {
    fn from(shape: Shape) -> Self {
        match shape {
            Shape::Circle => Self::rectangle(),
            Shape::Rect => Self::rectangle(),
            Shape::RoundedRect => Self::rectangle(),
            Shape::Triangle => Self::triangle(),
        }
    }
}

impl Indices<'_> {
    fn rectangle() -> Self {
        Self(&[0, 1, 2, 2, 3, 0])
    }

    fn triangle() -> Self {
        Self(&[4, 1, 2])
    }
}

// pub struct Vertices<'a>(&'a [Vector2<f32>]);

#[derive(Debug, Clone, Copy, Default)]
pub struct Attributes {
    pub pos: Vector2<u32>,
    pub dims: Size<u32>,
}

impl Attributes {
    pub fn new(dims: impl Into<Size<u32>>) -> Self {
        Self {
            pos: Vector2::default(),
            dims: dims.into(),
        }
    }

    pub fn new_with_pos(
        dims: impl Into<Size<u32>>,
        pos: impl Into<Vector2<u32>>
    ) -> Self {
        Self {
            pos: pos.into(),
            dims: dims.into(),
        }
    }

    // FIXME: maybe accuracy is important?
    pub fn adjust_ratio(&mut self, aspect_ratio: f32) {
        self.dims.width = (self.dims.height as f32 * aspect_ratio) as u32;
    }

    pub fn get_transform(&self, window_size: Size<u32>) -> Matrix4x4 {
        let mut matrix = Matrix4x4::IDENTITY;
        let ws: Size<f32> = window_size.into();
        let x = self.pos.x as f32 / ws.width * 2.0 - 1.0;
        let y = 1.0 - self.pos.y as f32 / ws.height * 2.0;
        let d: Size<f32> = self.dims.into();
        let scale = d / ws;
        matrix.transform(x, y, scale.width, scale.height);
        matrix
    }

    pub fn set_position(
        &mut self,
        new_pos: Vector2<f32>,
        transform: &mut Matrix4x4,
    ) {
        self.pos = new_pos.into();
        let x = self.pos.x as f32 / (self.dims.width as f32 / transform[0].x) * 2.0 - 1.0;
        let y = 1.0 - self.pos.y as f32 / (self.dims.height as f32 / transform[1].y) * 2.0;
        transform.translate(x, y);
    }
}

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

#[derive(Clone, Copy)]
pub struct Style {
    fill_color: Rgba<u8>,
    stroke_color: Rgba<u8>,
    dims: Size<u32>,
    shape: Shape,
    corners: Corners,
    rotate: f32,
    stroke_width: f32,
}

impl Style {
    pub fn new(
        fill_color: Rgba<u8>,
        dims: impl Into<Size<u32>>,
        shape: Shape,
    ) -> Self {
        Self {
            fill_color,
            stroke_color: Rgba::BLACK,
            dims: dims.into(),
            shape,
            corners: if shape.is_rounded_rect() { 0.025.into() } else { 0.0.into() },
            rotate: 0.0,
            stroke_width: 0.0,
        }
    }

    pub fn set_fill_color(&mut self, color: impl Into<Rgba<u8>>) {
        self.fill_color = color.into();
    }

    pub fn set_stroke_color(&mut self, color: impl Into<Rgba<u8>>) {
        self.stroke_color = color.into();
    }

    pub fn get_dimensions(&self) -> Size<u32> {
        self.dims
    }

    pub fn set_dimensions(&mut self, size: impl Into<Size<u32>>) {
        self.dims = size.into();
    }

    pub fn set_shape(&mut self, shape: Shape) {
        self.shape = shape;
    }

    pub fn set_corners<F: FnMut(&mut Corners)>(&mut self, mut f: F) {
        f(&mut self.corners);
    }

    pub fn set_rotation(&mut self, rotate: f32) {
        self.rotate = rotate;
    }

    pub fn set_stroke_width(&mut self, stroke: f32) {
        self.stroke_width = stroke;
    }
}

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
            fill_color: style.fill_color.into(),
            stroke_color: style.stroke_color.into(),
            shape: style.shape as u32,
            corners: style.corners,
            rotate: style.rotate,
            stroke_width: style.stroke_width,
            texture_id: -1,
            transform_id: 0,
        }
    }

    pub fn textured(style: &Style) -> Self {
        Self {
            fill_color: style.fill_color.into(),
            stroke_color: style.stroke_color.into(),
            shape: style.shape as u32,
            corners: style.corners,
            rotate: style.rotate,
            stroke_width: style.stroke_width,
            texture_id: 0,
            transform_id: 0,
        }
    }

    pub fn indices<'a>(&self) -> Indices<'a> {
        Indices::from(Shape::from(self.shape))
    }

    pub fn rgba_u8(&self) -> Rgba<u8> {
        self.fill_color.into()
    }

    pub fn set_fill_color<F: FnOnce(&mut Rgba<u8>)>(&mut self, f: F) {
        let mut rgba = self.fill_color.into();
        f(&mut rgba);
        self.fill_color = rgba.into();
    }

    pub fn revert_color(&mut self, cached_color: Rgba<u8>) {
        self.fill_color = cached_color.into();
    }


    pub fn is_hovered(&self, cursor: &Cursor, attr: &Attributes) -> bool {
        // let rotate = Matrix2x2::rotate(self.rotate);
        // let pos: Vector2<f32> = attr.pos.into();
        // let p = rotate * pos;
        let x = attr.pos.x as f32;
        let y = attr.pos.y as f32;

        let x_cursor = cursor.hover.pos.x;
        let y_cursor = cursor.hover.pos.y;

        let width = attr.dims.width as f32 / 2.0;
        let height = attr.dims.height as f32 / 2.0;

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

