use util::{tan, Matrix4x4, Size, Vector2};

use crate::{color::Rgba, cursor::Cursor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Rect,
    RoundedRect,
    Triangle,
}

impl Shape {
    pub(crate) fn is_triangle(&self) -> bool { matches!(self, Self::Triangle) }

    pub(crate) fn is_rounded_rect(&self) -> bool { matches!(self, Self::RoundedRect) }
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub h_align: HAlign,
    pub v_align: VAlign,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
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

#[derive(Default, Debug, Clone, Copy)]
pub struct Padding {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
}

impl Padding {
    pub(crate) fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self { top, bottom, left, right }
    }

    pub(crate) fn vertical(&self) -> u32 { self.top + self.bottom }

    pub(crate) fn horizontal(&self) -> u32 { self.left + self.right }

    pub(crate) fn top(&self) -> u32 { self.top }

    pub(crate) fn bottom(&self) -> u32 { self.bottom }

    pub(crate) fn left(&self) -> u32 { self.left }

    pub(crate) fn right(&self) -> u32 { self.right }

    pub fn set_top(&mut self, value: u32) { self.top = value }

    pub fn set_bottom(&mut self, value: u32) { self.bottom = value }

    pub fn set_left(&mut self, value: u32) { self.left = value }

    pub fn set_right(&mut self, value: u32) { self.right = value }

    pub fn set_all(&mut self, value: u32) {
        self.top = value;
        self.bottom = value;
        self.left = value;
        self.right = value;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pos: Vector2<u32>,
    size: Size<u32>,
    min_width: Option<u32>,
    min_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    fill_color: Rgba<u8>,
    hover_color: Option<Rgba<u8>>,
    click_color: Option<Rgba<u8>>,
    stroke_color: Rgba<u8>,
    shape: Shape,
    alignment: Alignment, // child alignment
    orientation: Orientation,
    corners: Corners,
    padding: Padding,
    spacing: u32,
    rotate: f32,
    stroke_width: f32,
}

impl Style {
    pub fn new(
        fill_color: Rgba<u8>,
        size: impl Into<Size<u32>>,
        shape: Shape,
    ) -> Self {
        Self {
            pos: Vector2::default(),
            size: size.into(),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            fill_color,
            hover_color: None,
            click_color: None,
            stroke_color: Rgba::BLACK,
            shape,
            alignment: Default::default(),
            orientation: Orientation::default(),
            corners: if shape.is_rounded_rect() { 0.025.into() } else { 0.0.into() },
            rotate: 0.0,
            stroke_width: 0.0,
            padding: Padding::default(),
            spacing: 0,
        }
    }

    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation
    }

    pub fn set_size(&mut self, size: impl Into<Size<u32>>) {
        self.size = size.into();
    }

    pub fn set_position(&mut self, pos: Vector2<u32>) {
        self.pos = pos;
    }

    pub(crate) fn set_transform(
        &mut self,
        new_pos: Vector2<f32>,
        transform: &mut Matrix4x4,
    ) {
        self.pos = new_pos.into();
        let x = self.pos.x as f32 / (self.size.width as f32 / transform[0].x) * 2.0 - 1.0;
        let y = 1.0 - self.pos.y as f32 / (self.size.height as f32 / transform[1].y) * 2.0;
        transform.translate(x, y);
    }

    pub fn set_min_width(&mut self, value: u32) { self.min_width = Some(value) }

    pub fn set_min_height(&mut self, value: u32) { self.min_height = Some(value) }

    pub fn set_max_width(&mut self, value: u32) { self.max_width = Some(value) }

    pub fn set_max_height(&mut self, value: u32) { self.max_height = Some(value) }

    pub fn set_fill_color(&mut self, color: impl Into<Rgba<u8>>) {
        self.fill_color = color.into();
    }

    pub fn set_hover_color(&mut self, color: impl Into<Rgba<u8>>) {
        self.hover_color = Some(color.into());
    }

    pub fn set_click_color(&mut self, color: impl Into<Rgba<u8>>) {
        self.click_color = Some(color.into());
    }

    pub fn set_stroke_color(&mut self, color: impl Into<Rgba<u8>>) {
        self.stroke_color = color.into();
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

    pub fn set_padding(&mut self, f: impl FnOnce(&mut Padding)) {
        f(&mut self.padding)
    }

    pub fn set_spacing(&mut self, spacing: u32) { self.spacing = spacing }

    pub(crate) fn adjust_ratio(&mut self, aspect_ratio: f32) {
        self.size.width = (self.size.height as f32 * aspect_ratio) as u32;
    }
}

impl Style {
    pub(crate) fn alignment(&self) -> Alignment { self.alignment }

    pub(crate) fn orientation(&self) -> Orientation { self.orientation }

    pub(crate) fn pos(&self) -> Vector2<u32> { self.pos }

    pub(crate) fn size(&self) -> Size<u32> { self.size }

    pub(crate) fn min_width(&self) -> Option<u32> { self.min_width }

    pub(crate) fn min_height(&self) -> Option<u32> { self.min_height }

    pub(crate) fn max_width(&self) -> Option<u32> { self.max_width }

    pub(crate) fn max_height(&self) -> Option<u32> { self.max_height }

    pub(crate) fn fill_color(&self) -> Rgba<u8> { self.fill_color }

    pub(crate) fn stroke_color(&self) -> Rgba<u8> { self.stroke_color }

    pub(crate) fn shape(&self) -> Shape { self.shape }

    pub(crate) fn corners(&self) -> Corners { self.corners }

    pub(crate) fn rotation(&self) -> f32 { self.rotate }

    pub(crate) fn stroke_width(&self) -> f32 { self.stroke_width }

    pub(crate) fn padding(&self) -> Padding { self.padding }

    pub(crate) fn spacing(&self) -> u32 { self.spacing }

    pub(crate) fn transform(&self, window_size: Size<u32>) -> Matrix4x4 {
        let mut matrix = Matrix4x4::IDENTITY;
        let ws: Size<f32> = window_size.into();
        let x = self.pos.x as f32 / ws.width * 2.0 - 1.0;
        let y = 1.0 - self.pos.y as f32 / ws.height * 2.0;
        let d: Size<f32> = self.size.into();
        let scale = d / ws;
        matrix.transform(x, y, scale.width, scale.height);
        matrix
    }

    pub(crate) fn is_hovered(&self, cursor: &Cursor) -> bool {
        // let rotate = Matrix2x2::rotate(self.rotate);
        // let pos: Vector2<f32> = attr.pos.into();
        // let p = rotate * pos;
        let x = self.pos.x as f32;
        let y = self.pos.y as f32;

        let x_cursor = cursor.hover.pos.x;
        let y_cursor = cursor.hover.pos.y;

        let width = self.size.width as f32 / 2.0;
        let height = self.size.height as f32 / 2.0;

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
