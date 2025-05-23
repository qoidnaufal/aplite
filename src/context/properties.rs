use aplite_types::{Fraction, Rect, Rgba, Size, Vector2};

use crate::context::layout::{Alignment, Orientation, Padding};
use aplite_renderer::{CornerRadius, Shape};
use aplite_renderer::RenderElementSource;

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined((u32, u32)),
    Source,
    Undefined,
}

#[derive(Debug, Clone, Copy)]
pub struct Properties {
    name: Option<&'static str>,
    rect: Rect<u32>,
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
    image_aspect_ratio: AspectRatio,
    rotation: f32,
    stroke_width: u32,
    texture_id: i32,
    dragable: bool,
}

impl Default for Properties {
    fn default() -> Self {
        Self::new()
    }
}

// internal data management
impl Properties {
    pub(crate) fn window_properties(size: Size<u32>) -> Self {
        Self {
            name: Some("ROOT"),
            rect: Rect::new((size.width() / 2, size.height() / 2), (size.width(), size.height())),
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
            image_aspect_ratio: AspectRatio::Undefined,
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
        self.rect.adjust_width(aspect_ratio);
    }

    pub(crate) fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
        self.rect.adjust_height(aspect_ratio);
    }

    pub(crate) fn is_hovered(&self, cursor: Vector2<f32>) -> bool {
        // FIXME: consider rotation
        // let rotate = Matrix2x2::rotate(self.rotate);
        // let pos: Vector2<f32> = attr.pos.into();
        // let p = rotate * pos;
        let x = self.rect.x() as f32;
        let y = self.rect.y() as f32;

        let x_cursor = cursor.x();
        let y_cursor = cursor.y();

        let width = self.rect.width() as f32 / 2.0;
        let height = self.rect.height() as f32 / 2.0;

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
            rect: Rect::new((0, 0), (0, 0)),
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
            image_aspect_ratio: AspectRatio::Undefined,
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
        self.rect.set_size(size.into())
    }

    pub fn set_position(&mut self, pos: Vector2<u32>) {
        self.rect.set_pos(pos);
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

    /// value is 0-100, with 100 is fully rounded
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

    pub(crate) fn set_image_aspect_ratio(&mut self, aspect_ratio: AspectRatio) {
        self.image_aspect_ratio = aspect_ratio;
    }
}

// getter
impl Properties {
    #[allow(unused)]
    pub(crate) fn name(&self) -> Option<&str> { self.name }

    pub(crate) fn alignment(&self) -> Alignment { self.alignment }

    pub(crate) fn orientation(&self) -> Orientation { self.orientation }

    pub(crate) fn rect(&self) -> Rect<u32> { self.rect }

    pub(crate) fn pos(&self) -> Vector2<u32> { self.rect.pos() }

    pub(crate) fn size(&self) -> Size<u32> { self.rect.size() }

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

    pub(crate) fn image_aspect_ratio(&self) -> AspectRatio { self.image_aspect_ratio }

    pub(crate) fn rotation(&self) -> f32 { self.rotation }

    pub(crate) fn stroke_width(&self) -> u32 { self.stroke_width }

    pub(crate) fn padding(&self) -> Padding { self.padding }

    pub(crate) fn spacing(&self) -> u32 { self.spacing }

    pub(crate) fn is_dragable(&self) -> bool { self.dragable }

    pub(crate) fn texture_id(&self) -> i32 { self.texture_id }
}

impl RenderElementSource for Properties {
    fn fill_color(&self) -> Rgba<f32> { self.fill_color().into() }

    fn stroke_color(&self) -> Rgba<f32> { self.stroke_color().into() }

    fn rect(&self) -> Rect<f32> { self.rect.into() }

    fn corners(&self) -> CornerRadius { self.corners() }

    fn shape(&self) -> Shape { self.shape() }

    fn rotation(&self) -> f32 { self.rotation() }

    fn stroke_width(&self) -> f32 { self.stroke_width() as _ }

    fn texture_id(&self) -> i32 { self.texture_id() }
}
