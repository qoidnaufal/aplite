use util::{tan, Matrix4x4, Size, Vector2};
use crate::context::Cursor;
use crate::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Circle,
    Rect,
    RoundedRect,
    Triangle,
}

impl ShapeKind {
    fn is_triangle(&self) -> bool { matches!(self, Self::Triangle) }

    fn is_rounded_rect(&self) -> bool { matches!(self, Self::RoundedRect) }
}

impl From<u32> for ShapeKind {
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

impl From<ShapeKind> for Indices<'_> {
    fn from(kind: ShapeKind) -> Self {
        match kind {
            ShapeKind::Circle => Self::rectangle(),
            ShapeKind::Rect => Self::rectangle(),
            ShapeKind::RoundedRect => Self::rectangle(),
            ShapeKind::Triangle => Self::triangle(),
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

#[derive(Debug, Clone, Default)]
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
        let x = (self.pos.x as f32 / ws.width - 0.5) * 2.0;
        let y = (0.5 - self.pos.y as f32 / ws.height) * 2.0;
        let d: Size<f32> = self.dims.into();
        let scale = d / ws;
        matrix.transform(x, y, scale.width, scale.height);
        matrix
    }

    pub fn set_position(
        &mut self,
        cursor: &Cursor,
        transform: &mut Matrix4x4,
    ) {
        let delta = cursor.hover.pos - cursor.click.delta;
        self.pos = delta.into();
        let x = (delta.x / (self.dims.width as f32 / transform[0].x) - 0.5) * 2.0;
        let y = (0.5 - delta.y / (self.dims.height as f32 / transform[1].y)) * 2.0;
        transform.translate(x, y);
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Radius {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
}

impl From<f32> for Radius {
    fn from(val: f32) -> Self {
        Self {
            top_left: val,
            bot_left: val,
            bot_right: val,
            top_right: val,
        }
    }
}

impl Radius {
    pub fn new_homogen(r: f32) -> Self {
        r.into()
    }

    pub fn set_all(&mut self, r: f32) {
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

pub struct Style {
    color: Rgba<u8>,
    outline: Rgba<u8>,
    dims: Size<u32>,
    kind: ShapeKind,
    radius: Radius,
    rotate: f32,
    stroke: f32,
}

impl Style {
    pub fn new(
        color: Rgba<u8>,
        dims: impl Into<Size<u32>>,
        kind: ShapeKind,
    ) -> Self {
        Self {
            color,
            outline: Rgba::BLACK,
            dims: dims.into(),
            kind,
            radius: if kind.is_rounded_rect() { 0.025.into() } else { 0.0.into() },
            rotate: 0.0,
            stroke: 0.0,
        }
    }

    pub fn set_fill(&mut self, color: impl Into<Rgba<u8>>) {
        self.color = color.into();
    }

    pub fn set_outline(&mut self, color: impl Into<Rgba<u8>>) {
        self.outline = color.into();
    }

    pub fn get_dimensions(&self) -> Size<u32> {
        self.dims
    }

    pub fn set_dimensions(&mut self, size: impl Into<Size<u32>>) {
        self.dims = size.into();
    }

    pub fn set_kind(&mut self, kind: ShapeKind) {
        self.kind = kind;
    }

    pub fn set_radius<F: FnMut(&mut Radius)>(&mut self, mut f: F) {
        f(&mut self.radius);
    }

    pub fn set_rotation(&mut self, rotate: f32) {
        self.rotate = rotate;
    }

    pub fn set_stroke(&mut self, stroke: f32) {
        self.stroke = stroke;
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Shape {
    color: Rgba<f32>,
    outline: Rgba<f32>,
    radius: Radius,
    kind: u32,
    rotate: f32,
    stroke: f32,
    pub texture_id: i32,
    pub transform_id: u32,
}

impl Shape {
    pub fn filled(style: &Style) -> Self {
        Self {
            color: style.color.into(),
            outline: style.outline.into(),
            kind: style.kind as u32,
            radius: style.radius,
            rotate: style.rotate,
            stroke: style.stroke,
            texture_id: -1,
            transform_id: 0,
        }
    }

    pub fn textured(style: &Style) -> Self {
        Self {
            color: style.color.into(),
            outline: style.outline.into(),
            kind: style.kind as u32,
            radius: style.radius,
            rotate: style.rotate,
            stroke: style.stroke,
            texture_id: 0,
            transform_id: 0,
        }
    }

    pub fn indices<'a>(&self) -> Indices<'a> {
        Indices::from(ShapeKind::from(self.kind))
    }

    pub fn rgba_u8(&self) -> Rgba<u8> {
        self.color.into()
    }

    pub fn set_color<F: FnOnce(&mut Rgba<u8>)>(&mut self, f: F) {
        let mut rgba = self.color.into();
        f(&mut rgba);
        self.color = rgba.into();
    }

    pub fn revert_color(&mut self, cached_color: Rgba<u8>) {
        self.color = cached_color.into();
    }


    pub fn is_hovered(&self, cursor: &Cursor, attr: &Attributes) -> bool {
        let x = attr.pos.x as f32;
        let y = attr.pos.y as f32;

        let x_cursor = cursor.hover.pos.x;
        let y_cursor = cursor.hover.pos.y;

        let width = attr.dims.width as f32 / 2.0;
        let height = attr.dims.height as f32 / 2.0;

        let angled = if ShapeKind::from(self.kind).is_triangle() {
            let c_tangen = tan(x_cursor - x, y_cursor - y + height);
            let t_tangen = tan(width / 2.0, height);
            (t_tangen - c_tangen).is_sign_negative()
        } else { true };

        (y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
            && angled
    }
}

