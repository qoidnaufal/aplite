use shared::{Fraction, Matrix4x4, Size, Vector2, Rgba};

use crate::context::layout::{Alignment, Orientation, Padding};
use crate::context::cursor::Cursor;
use crate::renderer::element::{CornerRadius, Shape};
use crate::renderer::util::RenderComponentSource;

#[derive(Debug, Clone, Copy)]
pub struct Properties {
    name: Option<&'static str>,
    pos: Vector2<u32>,
    size: Size<u32>,
    min_width: Option<u32>,
    min_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
    alignment: Alignment,
    orientation: Orientation,
    spacing: u32,
    padding: Padding,
    hover_color: Option<Rgba<u8>>,
    click_color: Option<Rgba<u8>>,
    fill_color: Rgba<u8>,
    stroke_color: Rgba<u8>,
    shape: Shape,
    corners: CornerRadius,
    rotation: f32,
    stroke_width: u32,
    texture_id: i32,
    dragable: bool,
}

// internal data management
impl Properties {
    pub(crate) fn window_properties(size: Size<u32>) -> Self {
        Self {
            name: Some("ROOT"),
            pos: (size / 2).into(),
            size,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            alignment: Default::default(),
            orientation: Default::default(),
            spacing: 10,
            padding: Default::default(),
            hover_color: None,
            click_color: None,
            fill_color: Rgba::BLACK,
            stroke_color: Rgba::BLACK,
            shape: Shape::Rect,
            corners: 0.into(),
            rotation: 0.0,
            stroke_width: 0,
            texture_id: -1,
            dragable: false,
        }
    }

    pub(crate) fn set_texture_id(&mut self, value: i32) {
        self.texture_id = value;
    }

    pub(crate) fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
        self.size.adjust_width(aspect_ratio);
    }

    #[allow(unused)]
    pub(crate) fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
        self.size.adjust_height(aspect_ratio);
    }

    // pub(crate) fn adjust_transform(&mut self, pos: Vector2<f32>, mat: &mut Matrix4x4) {
    //     let x = pos.x() / (self.size.width() as f32 / mat[0].x()) * 2.0 - 1.0;
    //     let y = 1.0 - pos.y() / (self.size.height() as f32 / mat[1].y()) * 2.0;
    //     self.set_position(pos.into());
    //     mat.with_translate(x, y);
    // }

    pub(crate) fn is_hovered(&self, cursor: &Cursor) -> bool {
        // FIXME: consider rotation
        // let rotate = Matrix2x2::rotate(self.rotate);
        // let pos: Vector2<f32> = attr.pos.into();
        // let p = rotate * pos;
        let x = self.pos.x() as f32;
        let y = self.pos.y() as f32;

        let x_cursor = cursor.hover.pos.x();
        let y_cursor = cursor.hover.pos.y();

        let width = self.size.width() as f32 / 2.0;
        let height = self.size.height() as f32 / 2.0;

        // let angled = if self.shape.is_triangle() {
        //     let c_tangen = tan(x_cursor - x, y_cursor - y + height);
        //     let t_tangen = tan(width / 2.0, height);
        //     (t_tangen - c_tangen).is_sign_negative()
        // } else { true };

        (y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
            // && angled
    }
}

// creation
impl Properties {
    pub const fn new() -> Self {
        Self {
            name: None,
            pos: Vector2::new(0, 0),
            size: Size::new(0, 0),
            min_width: Some(1),
            min_height: Some(1),
            max_width: None,
            max_height: None,
            alignment: Alignment::new(),
            orientation: Orientation::Vertical,
            spacing: 0,
            padding: Padding::new(5, 5, 5, 5),
            hover_color: None,
            click_color: None,
            fill_color: Rgba::DARK_GRAY,
            stroke_color: Rgba::WHITE,
            shape: Shape::RoundedRect,
            corners: CornerRadius::new_homogen(25),
            rotation: 0.0,
            stroke_width: 0,
            texture_id: -1,
            dragable: false,
        }
    }

    pub fn with_name(mut self, name: Option<&'static str>) -> Self {
        self.set_name(name);
        self
    }

    pub fn with_size(mut self, size: impl Into<Size<u32>>) -> Self {
        self.set_size(size);
        self
    }

    pub fn with_min_width(mut self, value: u32) -> Self {
        self.set_min_width(value);
        self
    }

    pub fn with_min_height(mut self, value: u32) -> Self {
        self.set_min_height(value);
        self
    }

    pub fn with_max_width(mut self, value: u32) -> Self {
        self.set_max_width(value);
        self
    }

    pub fn with_max_height(mut self, value: u32) -> Self {
        self.set_max_height(value);
        self
    }

    pub fn with_alignment(mut self, f: impl FnOnce(&mut Alignment)) -> Self {
        self.set_alignment(f);
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.set_orientation(orientation);
        self
    }

    pub fn with_spacing(mut self, value: u32) -> Self {
        self.set_spacing(value);
        self
    }

    pub fn with_padding(mut self, f: impl FnOnce(&mut Padding)) -> Self {
        self.set_padding(f);
        self
    }

    pub fn with_hover_color(mut self, color: Rgba<u8>) -> Self {
        self.set_hover_color(color);
        self
    }

    pub fn with_click_color(mut self, color: Rgba<u8>) -> Self {
        self.set_click_color(color);
        self
    }

    pub fn with_fill_color(mut self, color: Rgba<u8>) -> Self {
        self.set_fill_color(color);
        self
    }

    pub fn with_stroke_color(mut self, color: Rgba<u8>) -> Self {
        self.set_stroke_color(color);
        self
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.set_shape(shape);
        self
    }

    pub fn with_corners(mut self, f: impl FnOnce(&mut CornerRadius)) -> Self {
        self.set_corners(f);
        self
    }

    pub fn with_rotation(mut self, value: f32) -> Self {
        self.set_rotation(value);
        self
    }

    pub fn with_stroke_width(mut self, value: u32) -> Self {
        self.set_stroke_width(value);
        self
    }

    pub fn with_dragable(mut self, value: bool) -> Self {
        self.set_dragable(value);
        self
    }

    pub fn with_textured(mut self, value: bool) -> Self {
        let value = if value { 0 } else { -1 };
        self.set_texture_id(value);
        self
    }
}

// modifier
impl Properties {
    pub fn set_name(&mut self, name: Option<&'static str>) {
        self.name = name;
    }

    pub fn set_alignment(&mut self, f: impl FnOnce(&mut Alignment)) {
        f(&mut self.alignment)
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

    pub fn set_min_width(&mut self, value: u32) {
        self.min_width = Some(value)
    }

    pub fn set_min_height(&mut self, value: u32) {
        self.min_height = Some(value)
    }

    pub fn set_max_width(&mut self, value: u32) {
        self.max_width = Some(value)
    }

    pub fn set_max_height(&mut self, value: u32) {
        self.max_height = Some(value)
    }

    pub fn set_fill_color(&mut self, color: Rgba<u8>) {
        self.fill_color = color;
    }

    pub fn set_hover_color(&mut self, color: Rgba<u8>) {
        self.hover_color = Some(color);
    }

    pub fn set_click_color(&mut self, color: Rgba<u8>) {
        self.click_color = Some(color);
    }

    pub fn set_stroke_color(&mut self, color: Rgba<u8>) {
        self.stroke_color = color;
    }

    pub fn set_shape(&mut self, shape: Shape) {
        self.shape = shape;
    }

    pub fn set_corners<F: FnOnce(&mut CornerRadius)>(&mut self, f: F) {
        f(&mut self.corners);
    }

    pub fn set_rotation(&mut self, value: f32) {
        self.rotation = value;
    }

    pub fn set_stroke_width(&mut self, value: u32) {
        self.stroke_width = value;
    }

    pub fn set_padding(&mut self, f: impl FnOnce(&mut Padding)) {
        f(&mut self.padding)
    }

    pub fn set_spacing(&mut self, value: u32) {
        self.spacing = value
    }

    pub fn set_dragable(&mut self, value: bool) {
        self.dragable = value
    }
}

// getter
impl Properties {
    pub(crate) fn name(&self) -> Option<&str> { self.name }

    pub(crate) fn alignment(&self) -> Alignment { self.alignment }

    pub(crate) fn orientation(&self) -> Orientation { self.orientation }

    pub(crate) fn pos(&self) -> Vector2<u32> { self.pos }

    pub(crate) fn size(&self) -> Size<u32> { self.size }

    pub(crate) fn min_width(&self) -> Option<u32> { self.min_width }

    pub(crate) fn min_height(&self) -> Option<u32> { self.min_height }

    pub(crate) fn max_width(&self) -> Option<u32> { self.max_width }

    pub(crate) fn max_height(&self) -> Option<u32> { self.max_height }

    pub(crate) fn fill_color(&self) -> Rgba<u8> { self.fill_color }

    pub(crate) fn hover_color(&self) -> Option<Rgba<u8>> { self.hover_color }

    pub(crate) fn click_color(&self) -> Option<Rgba<u8>> { self.click_color }

    pub(crate) fn stroke_color(&self) -> Rgba<u8> { self.stroke_color }

    pub(crate) fn shape(&self) -> Shape { self.shape }

    pub(crate) fn corners(&self) -> CornerRadius { self.corners }

    pub(crate) fn rotation(&self) -> f32 { self.rotation }

    pub(crate) fn stroke_width(&self) -> u32 { self.stroke_width }

    pub(crate) fn padding(&self) -> Padding { self.padding }

    pub(crate) fn spacing(&self) -> u32 { self.spacing }

    pub(crate) fn is_dragable(&self) -> bool { self.dragable }

    pub(crate) fn texture_id(&self) -> i32 { self.texture_id }
}

impl RenderComponentSource for Properties {
    fn fill_color(&self) -> Rgba<f32> { self.fill_color().into() }

    fn stroke_color(&self) -> Rgba<f32> { self.stroke_color().into() }

    fn size(&self) -> Size<f32> { self.size.into() }

    fn corners(&self) -> CornerRadius { self.corners() }

    fn shape(&self) -> Shape { self.shape() }

    fn rotation(&self) -> f32 { self.rotation() }

    fn stroke_width(&self) -> f32 { self.stroke_width() as _ }

    fn texture_id(&self) -> i32 { self.texture_id() }
}
