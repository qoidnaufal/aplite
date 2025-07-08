use aplite_reactive::*;
use aplite_types::{Matrix3x2, Rect, Size, Vector2};

use crate::{context::layout::{Alignment, Orientation, Padding}, view::ViewId};

use super::cursor::Cursor;

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined((u32, u32)),
    Source,
    Undefined,
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetState {
    pub(crate) name: RwSignal<&'static str>,
    pub(crate) rect: RwSignal<Rect<u32>>,
    pub(crate) min_width: Option<u32>,
    pub(crate) min_height: Option<u32>,
    pub(crate) max_width: Option<u32>,
    pub(crate) max_height: Option<u32>,
    pub(crate) alignment: Alignment,
    pub(crate) orientation: Orientation,
    pub(crate) padding: Padding,
    pub(crate) spacing: u32,
    pub(crate) z_index: RwSignal<u32>,
    pub(crate) image_aspect_ratio: AspectRatio,
    pub(crate) dragable: RwSignal<bool>,
    pub(crate) hoverable: RwSignal<bool>,
    pub(crate) is_hovered: RwSignal<bool>,
    pub(crate) is_clicked: RwSignal<bool>,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self::new()
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(size: Size<u32>) -> Self {
        let x = size.width() / 2;
        let y = size.height() / 2;
        let w = size.width();
        let h = size.height();

        Self {
            name: RwSignal::new("Root"),
            rect: RwSignal::new(Rect::new((x, y), (w, h))),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            alignment: Default::default(),
            orientation: Default::default(),
            spacing: 10,
            z_index: RwSignal::new(0),
            padding: Default::default(),
            image_aspect_ratio: AspectRatio::Undefined,
            dragable: RwSignal::new(false),
            hoverable: RwSignal::new(false),
            is_hovered: RwSignal::new(false),
            is_clicked: RwSignal::new(false),
        }
    }

    // pub(crate) fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
    //     self.rect.adjust_width(aspect_ratio);
    // }

    // pub(crate) fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
    //     self.rect.adjust_height(aspect_ratio);
    // }

    pub(crate) fn detect_hover(&self, cursor: &mut Cursor, id: &ViewId) {
        // FIXME: consider rotation & maybe some precision
        // let rotate = Matrix2x2::rotate(self.rotate);
        // let pos: Vector2<f32> = attr.pos.into();
        // let p = rotate * pos;
        let pos = cursor.hover.pos;
        let rect = self.rect.get_untracked();
        let x = rect.x() as f32;
        let y = rect.y() as f32;

        let x_cursor = pos.x();
        let y_cursor = pos.y();

        let width = rect.width() as f32 / 2.0;
        let height = rect.height() as f32 / 2.0;

        // let angled = if self.shape.is_triangle() {
        //     let c_tangen = tan(x_cursor - x, y_cursor - y + height);
        //     let t_tangen = tan(width / 2.0, height);
        //     (t_tangen - c_tangen).is_sign_negative()
        // } else { true };

        self.is_hovered.set((y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
            && cursor.hover.z_index.get_untracked() <= self.z_index.get_untracked());

        if self.is_hovered.get_untracked() {
            cursor.hover.prev = cursor.hover.curr.replace(*id);
            cursor.hover.z_index.set(self.z_index.get_untracked());
        }
    }

    // pub(crate) fn toggle_click(&mut self) {
    //     self.is_clicked.update(|click| *click = !*click);
    // }
}

// creation
impl WidgetState {
    pub fn new() -> Self {
        Self {
            name: RwSignal::new(""),
            rect: RwSignal::new(Rect::new((0, 0), (0, 0))),
            min_width: Some(1),
            min_height: Some(1),
            max_width: None,
            max_height: None,
            alignment: Alignment::new(),
            orientation: Orientation::Vertical,
            spacing: 0,
            z_index: RwSignal::new(0),
            padding: Padding::new(5, 5, 5, 5),
            image_aspect_ratio: AspectRatio::Undefined,
            dragable: RwSignal::new(false),
            hoverable: RwSignal::new(false),
            is_hovered: RwSignal::new(false),
            is_clicked: RwSignal::new(false),
        }
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
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

    pub fn with_padding(mut self, value: Padding) -> Self {
        self.set_padding(value);
        self
    }

    // pub fn with_textured(mut self, value: bool) -> Self {
    //     let value = if value { 0 } else { -1 };
    //     self.set_texture_id(value);
    //     self
    // }
}

// modifier
impl WidgetState {
    pub fn set_name(&mut self, name: &'static str) {
        self.name.set(name);
    }

    pub fn set_alignment(&mut self, f: impl FnOnce(&mut Alignment)) {
        f(&mut self.alignment);
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    pub fn set_size(&mut self, size: impl Into<Size<u32>>) {
        self.rect.update(|rect| rect.set_size(size.into()))
    }

    pub fn set_position(&mut self, pos: Vector2<u32>) {
        self.rect.update(|rect| rect.set_pos(pos));
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

    pub fn set_padding(&mut self, value: Padding) {
        self.padding = value;
    }

    pub fn set_spacing(&mut self, value: u32) {
        self.spacing = value
    }

    pub(crate) fn set_image_aspect_ratio(&mut self, aspect_ratio: AspectRatio) {
        self.image_aspect_ratio = aspect_ratio;
    }
}

// getter
impl WidgetState {
    pub(crate) fn alignment(&self) -> Alignment { self.alignment }

    pub(crate) fn orientation(&self) -> Orientation { self.orientation }

    pub(crate) fn min_width(&self) -> Option<u32> { self.min_width }

    pub(crate) fn min_height(&self) -> Option<u32> { self.min_height }

    pub(crate) fn max_width(&self) -> Option<u32> { self.max_width }

    pub(crate) fn max_height(&self) -> Option<u32> { self.max_height }

    pub(crate) fn image_aspect_ratio(&self) -> AspectRatio { self.image_aspect_ratio }

    pub(crate) fn padding(&self) -> Padding { self.padding }

    pub(crate) fn spacing(&self) -> u32 { self.spacing }

    pub(crate) fn get_transform(&self, screen: Size<f32>) -> Matrix3x2 {
        let rect = self.rect.read_untracked(|rect| rect.f32());
        let size = rect.size();
        let x = rect.x() / screen.width() * 2.0 - 1.0;
        let y = 1.0 - rect.y() / screen.height() * 2.0;
        let scale = size / screen;

        Matrix3x2::IDENTITY
            .with_translate(x, y)
            .with_scale(scale.width(), scale.height())
    }

    // pub(crate) fn texture_id(&self) -> i32 { self.texture_id }
}

// impl RenderElementSource for Properties {
//     fn fill_color(&self) -> Rgba<f32> { self.fill_color().into() }

//     fn stroke_color(&self) -> Rgba<f32> { self.stroke_color().into() }

//     fn rect(&self) -> Rect<f32> { self.rect.into() }

//     fn corners(&self) -> CornerRadius { self.corners() }

//     fn shape(&self) -> Shape { self.shape() }

//     fn rotation(&self) -> f32 { self.rotation() }

//     fn stroke_width(&self) -> f32 { self.stroke_width() as _ }
// }
