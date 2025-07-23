use aplite_reactive::*;
use aplite_types::{Matrix3x2, Rect, Size, Vec2f};

use crate::context::layout::{Alignment, AlignV, Orientation, Padding};
use crate::context::cursor::Cursor;
use crate::view::ViewId;

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined((u32, u32)),
    Source,
    Undefined,
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetState {
    pub(crate) name: &'static str,
    pub(crate) root_id: RwSignal<Option<ViewId>>,
    pub(crate) rect: RwSignal<Rect>,
    pub(crate) min_width: Option<f32>,
    pub(crate) min_height: Option<f32>,
    pub(crate) max_width: Option<f32>,
    pub(crate) max_height: Option<f32>,
    pub(crate) alignment: Alignment,
    pub(crate) orientation: Orientation,
    pub(crate) padding: Padding,
    pub(crate) spacing: f32,
    pub(crate) z_index: RwSignal<u32>,
    pub(crate) image_aspect_ratio: AspectRatio,
    pub(crate) dragable: RwSignal<bool>,
    pub(crate) hoverable: RwSignal<bool>,
    pub(crate) is_hovered: RwSignal<bool>,
    pub(crate) is_clicked: RwSignal<bool>,
    pub(crate) trigger_callback: RwSignal<bool>,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            name: "",
            root_id: RwSignal::new(None),
            rect: RwSignal::new(Rect::new(0., 0., 0., 0.)),
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            alignment: Alignment::new(),
            orientation: Orientation::Vertical,
            spacing: 0.,
            z_index: RwSignal::new(0),
            padding: Padding::default(),
            image_aspect_ratio: AspectRatio::Undefined,
            dragable: RwSignal::new(false),
            hoverable: RwSignal::new(false),
            is_hovered: RwSignal::new(false),
            is_clicked: RwSignal::new(false),
            trigger_callback: RwSignal::new(false),
        }
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(size: Size) -> Self {
        let x = 0.;
        let y = 0.;
        let w = size.width;
        let h = size.height;

        Self::new()
            .with_name("Root")
            .with_alignment(|alignment| alignment.set_v(AlignV::Top))
            .with_size((w, h))
            .with_position((x, y))
    }

    // pub(crate) fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
    //     self.rect.adjust_width(aspect_ratio);
    // }

    // pub(crate) fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
    //     self.rect.adjust_height(aspect_ratio);
    // }

    // FIXME: consider rotation & maybe some precision
    pub(crate) fn detect_hover(&self, cursor: &Cursor) -> bool {
        let rect = self.rect.get_untracked();

        let is_hovered = rect.contains(cursor.hover.pos)
            && cursor.hover.z_index <= self.z_index.get_untracked();

        self.is_hovered.set(is_hovered);

        is_hovered
    }
}

// creation
impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.set_name(name);
        self
    }

    pub(crate) fn with_position(self, pos: impl Into<Vec2f>) -> Self {
        self.rect.update_untracked(|rect| rect.set_pos(pos.into()));
        self
    }

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

    pub fn with_alignment(mut self, f: impl FnOnce(&mut Alignment)) -> Self {
        self.set_alignment(f);
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
}

// modifier
impl WidgetState {
    pub fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }

    pub fn set_alignment(&mut self, f: impl FnOnce(&mut Alignment)) {
        f(&mut self.alignment);
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    pub fn set_size(&mut self, size: impl Into<Size>) {
        self.rect.update(|rect| rect.set_size(size.into()))
    }

    pub fn set_position(&mut self, pos: Vec2f) {
        self.rect.update(|rect| rect.set_pos(pos));
    }

    pub fn set_min_width(&mut self, value: f32) {
        self.min_width = Some(value)
    }

    pub fn set_min_height(&mut self, value: f32) {
        self.min_height = Some(value)
    }

    pub fn set_max_width(&mut self, value: f32) {
        self.max_width = Some(value)
    }

    pub fn set_max_height(&mut self, value: f32) {
        self.max_height = Some(value)
    }

    pub fn set_padding(&mut self, value: Padding) {
        self.padding = value;
    }

    pub fn set_spacing(&mut self, value: f32) {
        self.spacing = value
    }

    pub fn set_image_aspect_ratio(&mut self, aspect_ratio: AspectRatio) {
        self.image_aspect_ratio = aspect_ratio;
    }
}

// getter
impl WidgetState {
    pub(crate) fn alignment(&self) -> Alignment { self.alignment }

    pub(crate) fn orientation(&self) -> Orientation { self.orientation }

    pub(crate) fn image_aspect_ratio(&self) -> AspectRatio { self.image_aspect_ratio }

    pub(crate) fn get_transform(&self, screen: Size) -> Matrix3x2 {
        let rect = self.rect.get_untracked();
        let tx = rect.center_x() / screen.width * 2.0 - 1.0;
        let ty = 1.0 - rect.center_y() / screen.height * 2.0;
        let sx = rect.width / screen.width;
        let sy = rect.height / screen.height;

        Matrix3x2::from_scale_translate(sx, sy, tx, ty)
    }
}
