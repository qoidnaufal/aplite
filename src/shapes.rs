use util::{tan, Matrix4x4, Size, Vector2};
use crate::context::Cursor;
use crate::color::Rgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    FilledTriangle,
    FilledRectangle,
    TexturedRectangle,
}

impl ShapeKind {
    pub fn is_triangle(&self) -> bool { matches!(self, Self::FilledTriangle) }
}

impl From<u32> for ShapeKind {
    fn from(num: u32) -> Self {
        match num {
            0 => Self::FilledTriangle,
            1 => Self::FilledRectangle,
            2 => Self::TexturedRectangle,
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
            ShapeKind::FilledTriangle => Self::triangle(),
            ShapeKind::FilledRectangle => Self::rectangle(),
            ShapeKind::TexturedRectangle => Self::rectangle(),
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

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Shape {
    pub pos: Vector2<u32>,
    pub dimensions: Size<u32>,
    pub color: Rgb<f32>,
    pub kind: u32,
}

impl Shape {
    pub fn filled(color: Rgb<u8>, kind : ShapeKind, size: impl Into<Size<u32>>) -> Self {
        Self {
            pos: Vector2::default(),
            dimensions: size.into(),
            color: color.into(),
            kind: kind as u32,
        }
    }

    pub fn textured(kind: ShapeKind) -> Self {
        Self {
            pos: Vector2::default(),
            dimensions: Size::new(500, 500),
            color: Rgb::WHITE.into(),
            kind: kind as u32,
        }
    }

    pub fn indices<'a>(&self) -> Indices<'a> {
        Indices::from(ShapeKind::from(self.kind))
    }

    pub fn set_color<F: FnOnce(&mut Rgb<u8>)>(&mut self, f: F) {
        let mut rgb = self.color.into();
        f(&mut rgb);
        self.color = rgb.into();
    }

    pub fn revert_color(&mut self, cached_color: Rgb<u8>) {
        self.color = cached_color.into();
    }

    pub fn get_transform(&self, window_size: Size<u32>) -> Matrix4x4 {
        let mut matrix = Matrix4x4::IDENTITY;
        let ws: Size<f32> = window_size.into();
        let x = (self.pos.x as f32 / ws.width - 0.5) * 2.0;
        let y = (0.5 - self.pos.y as f32 / ws.height) * 2.0;
        let d: Size<f32> = self.dimensions.into();
        let scale = d / ws;
        matrix.transform(x, y, scale.width, scale.height);
        matrix
    }

    pub fn is_hovered(&self, cursor: &Cursor) -> bool {
        let x = self.pos.x as f32;
        let y = self.pos.y as f32;

        let x_cursor = cursor.hover.pos.x;
        let y_cursor = cursor.hover.pos.y;

        let width = self.dimensions.width as f32 / 2.0;
        let height = self.dimensions.height as f32 / 2.0;

        let angled = if ShapeKind::from(self.kind).is_triangle() {
            let c_tangen = tan(x_cursor - x, y_cursor - y + height);
            let t_tangen = tan(width / 2.0, height);
            (t_tangen - c_tangen).is_sign_negative()
        } else { true };

        (y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
            && angled
    }

    pub fn set_position(
        &mut self,
        cursor: &Cursor,
        transform: &mut Matrix4x4,
    ) {
        let delta = cursor.hover.pos - cursor.click.delta;
        self.pos = delta.into();
        let x = (delta.x / (self.dimensions.width as f32 / transform[0].x) - 0.5) * 2.0;
        let y = (0.5 - delta.y / (self.dimensions.height as f32 / transform[1].y)) * 2.0;
        transform.translate(x, y);
    }
}

