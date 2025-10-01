use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::Shape;
use aplite_types::{
    Matrix3x2,
    // Rect,
    // Size,
    CornerRadius,
    Paint,
    Rgba,
    Unit,
};

use crate::widget::WidgetId;
use crate::layout::{AlignV, AlignH, Orientation, Padding, LayoutRules};

thread_local! {
    pub(crate) static NODE_STORAGE: RefCell<HashMap<WidgetId, Rc<RefCell<WidgetState>>>> =
        RefCell::new(HashMap::with_capacity(1024));
}

#[derive(Clone)]
pub struct WidgetState {
    pub(crate) width: Unit,
    pub(crate) height: Unit,
    pub(crate) transform: Matrix3x2,

    pub(crate) shape: Shape,
    pub(crate) corner_radius: CornerRadius,
    pub(crate) flag: Flag,

    pub(crate) background_paint: Paint,
    pub(crate) aspect_ratio: AspectRatio,

    pub(crate) border_width: u32,
    pub(crate) border_paint: Paint,

    pub(crate) padding: Padding,
    pub(crate) spacing: u8,
    pub(crate) orientation: Orientation,
    pub(crate) align_v: AlignV,
    pub(crate) align_h: AlignH,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            width: Unit::Fixed(1.),
            height: Unit::Fixed(1.),
            transform: Matrix3x2::identity(),

            shape: Shape::Rect,
            corner_radius: CornerRadius::splat(0),
            flag: Flag::default(),

            background_paint: Paint::Color(Rgba::RED),
            aspect_ratio: AspectRatio::Undefined,

            border_paint: Paint::Color(Rgba::WHITE),
            border_width: 0,

            align_v: AlignV::Top,
            align_h: AlignH::Left,
            orientation: Orientation::Vertical,
            spacing: 0,
            padding: Padding::default(),
        }
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(width: f32, height: f32) -> Self {
        Self {
            width: Unit::Fixed(width),
            height: Unit::Fixed(height),
            background_paint: Paint::Color(Rgba::TRANSPARENT),
            border_paint: Paint::Color(Rgba::TRANSPARENT),
            ..Default::default()
        }
    }

    pub(crate) fn layout_rules(&self) -> LayoutRules {
        LayoutRules {
            orientation: self.orientation,
            align_h: self.align_h,
            align_v: self.align_v,
            padding: self.padding,
            spacing: self.spacing,
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.flag.is_visible()
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, width: impl Into<Unit>, height: impl Into<Unit>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            ..self
        }
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(self, paint: impl Into<Paint>) -> Self {
        Self {
            background_paint: paint.into(),
            ..self
        }
    }

    pub fn with_aspect_ratio(self, aspect_ratio: AspectRatio) -> Self {
        Self {
            aspect_ratio,
            ..self
        }
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(self, paint: impl Into<Paint>) -> Self {
        Self {
            border_paint: paint.into(),
            ..self
        }
    }

    pub fn with_border_width(self, val: u32) -> Self {
        Self {
            border_width: val,
            ..self
        }
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        Self {
            shape,
            ..self
        }
    }

    pub fn with_rotation_deg(mut self, deg: f32) -> Self {
        self.transform.set_rotate_deg(deg);
        self
    }

    pub fn with_rotation_rad(mut self, rad: f32) -> Self {
        self.transform.set_rotate_rad(rad);
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        Self {
            corner_radius: val,
            ..self
        }
    }

    pub fn with_align_h(self, align_h: AlignH) -> Self {
        Self {
            align_h,
            ..self
        }
    }

    pub fn with_align_v(self, align_v: AlignV) -> Self {
        Self {
            align_v,
            ..self
        }
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        Self {
            orientation,
            ..self
        }
    }

    pub fn with_padding(self, padding: Padding) -> Self {
        Self {
            padding,
            ..self
        }
    }

    pub fn with_spacing(self, spacing: u8) -> Self {
        Self {
            spacing,
            ..self
        }
    }

    pub fn hoverable(mut self) -> Self {
        self.flag.set_hoverable(true);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined(u8, u8),
    Source,
    Undefined,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Flag(u8);

impl Default for Flag {
    fn default() -> Self {
        Self(Self::DIRTY)
    }
}

impl Flag {
    const HIDE: u8 = 1;
    const DRAGABLE: u8 = 2;
    const HOVERABLE: u8 = 4;
    const DIRTY: u8 = 8;
    const NEEDS_LAYOUT: u8 = 16;

    #[inline(always)]
    pub(crate) fn is_hidden(&self) -> bool {
        self.0 & Self::HIDE == Self::HIDE
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.0 & Self::HIDE != Self::HIDE
    }

    #[inline(always)]
    pub(crate) fn is_dragable(&self) -> bool {
        self.0 & Self::DRAGABLE == Self::DRAGABLE
    }

    #[inline(always)]
    pub(crate) fn is_hoverable(&self) -> bool {
        self.0 & Self::HOVERABLE == Self::HOVERABLE
    }

    #[inline(always)]
    pub(crate) fn is_dirty(&self) -> bool {
        self.0 & Self::DIRTY == Self::DIRTY
    }

    #[inline(always)]
    pub(crate) fn needs_layout(&self) -> bool {
        self.0 & Self::NEEDS_LAYOUT == Self::NEEDS_LAYOUT
    }

    #[inline(always)]
    pub(crate) fn toggle_hidden(&mut self) {
        self.0 ^= Self::HIDE;
    }

    #[inline(always)]
    pub(crate) fn toggle_draggable(&mut self) {
        self.0 ^= Self::DRAGABLE;
    }

    #[inline(always)]
    pub(crate) fn toggle_hoverable(&mut self) {
        self.0 ^= Self::HOVERABLE;
    }

    #[inline(always)]
    pub(crate) fn toggle_dirty(&mut self) {
        self.0 ^= Self::DIRTY;
    }

    #[inline(always)]
    pub(crate) fn toggle_needs_layout(&mut self) {
        self.0 ^= Self::NEEDS_LAYOUT
    }

    #[inline(always)]
    pub(crate) fn set_hidden(&mut self, hidden: bool) {
        if self.is_hidden() ^ hidden {
            self.toggle_hidden();
        }
    }

    #[inline(always)]
    pub(crate) fn set_dragable(&mut self, dragable: bool) {
        if self.is_dragable() ^ dragable {
            self.toggle_draggable();
        }
    }

    #[inline(always)]
    pub(crate) fn set_hoverable(&mut self, hoverable: bool) {
        if self.is_hoverable() ^ hoverable {
            self.toggle_hoverable();
        }
    }

    #[inline(always)]
    pub(crate) fn set_dirty(&mut self, dirty: bool) {
        if self.is_dirty() ^ dirty {
            self.toggle_dirty();
        }
    }

    #[inline(always)]
    pub(crate) fn set_needs_layout(&mut self, needs_layout: bool) {
        if self.needs_layout() ^ needs_layout {
            self.toggle_needs_layout();
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeRef {
    node: Weak<RefCell<WidgetState>>,
}

impl NodeRef {
    #[inline(always)]
    pub(crate) fn try_upgrade(&self) -> Option<Rc<RefCell<WidgetState>>> {
        self.node.upgrade()
    }

    pub(crate) fn upgrade(&self) -> Rc<RefCell<WidgetState>> {
        self.try_upgrade().unwrap()
    }

    pub fn set_color(&self, color: Rgba) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().background_paint = color.into();
            node.borrow_mut().flag.set_dirty(true);
        }
    }

    pub fn set_border_color(&self, border_color: Rgba) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().background_paint = border_color.into();
            node.borrow_mut().flag.set_dirty(true);
        }
    }

    pub fn set_border_width(&self, val: u32) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().border_width = val;
            node.borrow_mut().flag.set_dirty(true);
        }
    }

    pub fn set_corner_radius(&self, corner_radius: CornerRadius) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().corner_radius = corner_radius;
            node.borrow_mut().flag.set_dirty(true);
        }
    }

    pub fn set_shape(&self, shape: Shape) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().shape = shape;
            node.borrow_mut().flag.set_dirty(true);
        }
    }

    pub fn set_rotation_deg(&self, deg: f32) {
        self.set_rotation_rad(deg.to_radians());
    }

    pub fn set_rotation_rad(&self, rad: f32) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().transform.set_rotate_rad(rad);
            node.borrow_mut().flag.set_dirty(true);
        }
    }

    pub fn set_spacing(&self, val: u8) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().spacing = val;
            node.borrow_mut().flag.set_needs_layout(true);
        }
    }

    pub fn toggle_hoverable(&self) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().flag.toggle_hoverable();
        }
    }

    pub fn toggle_dragable(&self) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().flag.toggle_draggable();
        }
    }

    pub fn hide(&self, val: bool) {
        if let Some(node) = self.try_upgrade() {
            let prev = node.borrow().flag.is_hidden();
            node.borrow_mut().flag.set_hidden(val);
            if prev != val {
                node.borrow_mut().flag.set_needs_layout(true);
            }
        }
    }

    pub fn set_image_aspect_ratio(&self, val: AspectRatio) {
        if let Some(node) = self.try_upgrade() {
            node.borrow_mut().aspect_ratio = val;
            node.borrow_mut().flag.set_dirty(true);
        }
    }
}
