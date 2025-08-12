use aplite_renderer::Shape;
use aplite_types::{
    Matrix3x2,
    Rect,
    Size,
    Vec2f,
    CornerRadius,
    Paint,
    Rgba,
};

use crate::context::layout::{AlignV, AlignH, Orientation, Padding};
use crate::context::cursor::Cursor;
use crate::widget::WidgetEvent;

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined((u32, u32)),
    Source,
    Undefined,
}

#[derive(Clone)]
pub struct WidgetState {
    pub(crate) name: &'static str,
    pub(crate) rect: Rect,
    pub(crate) rotation: f32, // in radians
    // pub(crate) transform: Matrix3x2,
    pub(crate) min_width: Option<f32>,
    pub(crate) min_height: Option<f32>,
    pub(crate) max_width: Option<f32>,
    pub(crate) max_height: Option<f32>,
    pub(crate) align_v: AlignV,
    pub(crate) align_h: AlignH,
    pub(crate) orientation: Orientation,
    pub(crate) padding: Padding,
    pub(crate) spacing: f32,
    // pub(crate) z_index: u32,
    pub(crate) image_aspect_ratio: AspectRatio,
    pub(crate) shape: Shape,
    pub(crate) corner_radius: CornerRadius,
    pub(crate) border_width: f32,
    pub(crate) event: Option<WidgetEvent>,
    pub(crate) background_paint: Paint,
    pub(crate) border_paint: Paint,
    pub(crate) dragable: bool,
    pub(crate) hoverable: bool,
}

impl std::fmt::Debug for WidgetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            name: "",
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            rotation: 0.0,
            // transform: Matrix3x2::identity(),
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            align_v: AlignV::Top,
            align_h: AlignH::Left,
            orientation: Orientation::Vertical,
            spacing: 0.,
            // z_index: 0,
            padding: Padding::default(),
            image_aspect_ratio: AspectRatio::Undefined,
            dragable: false,
            hoverable: false,
            shape: Shape::Rect,
            corner_radius: CornerRadius::splat(0.0),
            event: None,
            background_paint: Paint::Color(Rgba::RED),
            border_paint: Paint::Color(Rgba::WHITE),
            border_width: 0.0,
        }
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(size: Size) -> Self {
        Self {
            name: "Root",
            rect: Rect::from_size(size),
            align_h: AlignH::Center,
            background_paint: Paint::Color(Rgba::TRANSPARENT),
            border_paint: Paint::Color(Rgba::TRANSPARENT),
            ..Default::default()
        }
    }

    // FIXME: consider rotation & maybe some precision
    // pub(crate) fn detect_hover(&self, cursor: &Cursor) -> bool {
    //     self.rect.contains(cursor.hover.pos)
    // }

    pub(crate) fn get_transform(&self, screen: Size) -> Matrix3x2 {
        let rect = self.rect;
        let tx = rect.center_x() / screen.width * 2.0 - 1.0;
        let ty = 1.0 - rect.center_y() / screen.height * 2.0;
        let sx = rect.width / screen.width;
        let sy = rect.height / screen.height;

        Matrix3x2::from_scale_translate(sx, sy, tx, ty)
    }
}

// builder
impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.set_name(name);
        self
    }

    #[allow(unused)]
    /// Types which implement [`Into<Vec2f>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Vec2f`](aplite_types::Vec2f)
    /// - [`Vec2u`](aplite_types::Vec2u)
    pub(crate) fn with_position(mut self, pos: impl Into<Vec2f>) -> Self {
        self.rect.set_pos(pos.into());
        self
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.set_size(size);
        self
    }

    pub fn with_min_width(mut self, value: f32) -> Self {
        self.set_min_width(value);
        self
    }

    pub fn with_min_height(mut self, value: f32) -> Self {
        self.set_min_height(value);
        self
    }

    pub fn with_max_width(mut self, value: f32) -> Self {
        self.set_max_width(value);
        self
    }

    pub fn with_max_height(mut self, value: f32) -> Self {
        self.set_max_height(value);
        self
    }

    pub fn with_align_h(mut self, align_h: AlignH) -> Self {
        self.set_align_h(align_h);
        self
    }

    pub fn with_align_v(mut self, align_v: AlignV) -> Self {
        self.set_align_v(align_v);
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.set_orientation(orientation);
        self
    }

    pub fn with_spacing(mut self, value: f32) -> Self {
        self.set_spacing(value);
        self
    }

    pub fn with_padding(mut self, value: Padding) -> Self {
        self.set_padding(value);
        self
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.set_shape(shape);
        self
    }

    pub fn with_corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.set_corner_radius(corner_radius);
        self
    }

    pub fn with_rotation_deg(mut self, deg: f32) -> Self {
        self.set_rotation_deg(deg);
        self
    }

    pub fn with_rotation_rad(mut self, rad: f32) -> Self {
        self.set_rotation_rad(rad);
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background(mut self, paint: impl Into<Paint>) -> Self {
        self.set_background(paint);
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_color(mut self, color: impl Into<Paint>) -> Self {
        self.set_border_color(color);
        self
    }

    pub fn with_dragable(mut self) -> Self {
        self.set_dragable(true);
        self
    }

    pub fn with_border_width(mut self, val: f32) -> Self {
        self.border_width = val;
        self
    }
}

// modifier
impl WidgetState {
    #[inline(always)]
    pub fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }

    #[inline(always)]
    pub fn set_align_h(&mut self, align_h: AlignH) {
        self.align_h = align_h;
    }

    #[inline(always)]
    pub fn set_align_v(&mut self, align_v: AlignV) {
        self.align_v = align_v;
    }

    #[inline(always)]
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    #[inline(always)]
    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn set_size(&mut self, size: impl Into<Size>) {
        self.rect.set_size(size.into())
    }

    #[inline(always)]
    pub fn set_position(&mut self, pos: Vec2f) {
        self.rect.set_pos(pos);
    }

    #[inline(always)]
    pub fn set_min_width(&mut self, value: f32) {
        self.min_width = Some(value)
    }

    #[inline(always)]
    pub fn set_min_height(&mut self, value: f32) {
        self.min_height = Some(value)
    }

    #[inline(always)]
    pub fn set_max_width(&mut self, value: f32) {
        self.max_width = Some(value)
    }

    #[inline(always)]
    pub fn set_max_height(&mut self, value: f32) {
        self.max_height = Some(value)
    }

    #[inline(always)]
    pub fn set_padding(&mut self, value: Padding) {
        self.padding = value;
    }

    #[inline(always)]
    pub fn set_spacing(&mut self, value: f32) {
        self.spacing = value
    }

    #[inline(always)]
    pub fn set_image_aspect_ratio(&mut self, aspect_ratio: AspectRatio) {
        self.image_aspect_ratio = aspect_ratio;
    }

    #[inline(always)]
    pub fn set_shape(&mut self, shape: Shape) {
        self.shape = shape;
    }

    #[inline(always)]
    pub fn set_corner_radius(&mut self, corner_radius: CornerRadius) {
        self.corner_radius = corner_radius;
    }

    #[inline(always)]
    pub fn set_rotation_deg(&mut self, deg: f32) {
        self.rotation = deg.to_radians();
    }

    #[inline(always)]
    pub fn set_rotation_rad(&mut self, rad: f32) {
        self.rotation = rad;
    }

    #[inline(always)]
    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn set_background(&mut self, paint: impl Into<Paint>) {
        self.background_paint = paint.into();
    }

    #[inline(always)]
    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn set_border_color(&mut self, color: impl Into<Paint>) {
        self.border_paint = color.into();
    }

    #[inline(always)]
    pub fn set_dragable(&mut self, drag: bool) {
        self.dragable = drag;
    }
}
