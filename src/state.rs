use std::rc::{Rc, Weak};
use std::cell::RefCell;

use aplite_renderer::Shape;
use aplite_storage::{IndexMap, Entity, entity};
use aplite_types::{
    Matrix3x2,
    Rect,
    Size,
    CornerRadius,
    Paint,
    Rgba,
};

use crate::layout::{AlignV, AlignH, Orientation, Padding};

entity! {
    pub WidgetId
}

thread_local! {
    pub(crate) static NODE_STORAGE: RefCell<IndexMap<WidgetId, Rc<RefCell<WidgetState>>>> =
        RefCell::new(IndexMap::with_capacity(1024));
}

pub struct WidgetState {
    pub(crate) rect: Rect,
    // pub(crate) rotation: f32, // in radians
    pub(crate) transform: Matrix3x2,
    pub(crate) min_width: Option<f32>,
    pub(crate) min_height: Option<f32>,
    pub(crate) max_width: Option<f32>,
    pub(crate) max_height: Option<f32>,
    pub(crate) align_v: AlignV,
    pub(crate) align_h: AlignH,
    pub(crate) orientation: Orientation,
    pub(crate) padding: Padding,
    pub(crate) spacing: u8,
    // pub(crate) z_index: u8,
    pub(crate) image_aspect_ratio: AspectRatio,
    pub(crate) shape: Shape,
    pub(crate) corner_radius: CornerRadius,
    pub(crate) border_width: f32,
    pub(crate) background_paint: Paint,
    pub(crate) border_paint: Paint,
    pub(crate) flag: Flag,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            // rotation: 0.0,
            transform: Matrix3x2::identity(),
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            align_v: AlignV::Top,
            align_h: AlignH::Left,
            orientation: Orientation::Vertical,
            spacing: 0,
            // z_index: 0,
            padding: Padding::default(),
            image_aspect_ratio: AspectRatio::Undefined,
            shape: Shape::Rect,
            corner_radius: CornerRadius::splat(0),
            background_paint: Paint::Color(Rgba::RED),
            border_paint: Paint::Color(Rgba::WHITE),
            border_width: 0.0,
            flag: Flag::default(),
            // update: None,
        }
    }
}

// internal data management
impl WidgetState {
    pub(crate) fn window(rect: Rect) -> Self {
        Self {
            rect,
            background_paint: Paint::Color(Rgba::TRANSPARENT),
            border_paint: Paint::Color(Rgba::TRANSPARENT),
            ..Default::default()
        }
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
    id: WidgetId,
}

impl Default for NodeRef {
    fn default() -> Self {
        let state = Rc::new(RefCell::new(WidgetState::default()));
        let node = Rc::downgrade(&state);
        let id = NODE_STORAGE.with_borrow_mut(|s| s.insert(state));

        Self { node, id }
    }
}

impl NodeRef {
    pub(crate) fn window(rect: Rect) -> Self {
        let state = Rc::new(RefCell::new(WidgetState::window(rect)));
        let node = Rc::downgrade(&state);
        let id = NODE_STORAGE.with_borrow_mut(|s| s.insert(state));

        Self { node, id }
    }

    pub(crate) fn id(&self) -> WidgetId {
        self.id
    }

    #[inline(always)]
    pub(crate) fn try_upgrade(&self) -> Option<Rc<RefCell<WidgetState>>> {
        self.node.upgrade()
    }

    pub(crate) fn upgrade(&self) -> Rc<RefCell<WidgetState>> {
        self.try_upgrade().unwrap()
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.try_upgrade()
            .is_some_and(|state| {
                !state.borrow().flag.is_hidden()
            })
    }

    pub(crate) fn is_hoverable(&self) -> bool {
        self.try_upgrade()
            .is_some_and(|state| state.borrow().flag.is_hoverable())
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, size: impl Into<Size>) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().rect.set_size(size.into());
        }
        self
    }

    pub fn with_min_width(self, val: f32) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().min_width = Some(val);
        }
        self
    }

    pub fn with_max_width(self, val: f32) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().max_width = Some(val);
        }
        self
    }

    pub fn with_min_height(self, val: f32) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().min_height = Some(val);
        }
        self
    }

    pub fn with_max_height(self, val: f32) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().max_height = Some(val);
        }
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(self, paint: impl Into<Paint>) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().background_paint = paint.into();
        }
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(self, color: impl Into<Paint>) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().border_paint = color.into();
        }
        self
    }

    pub fn with_stroke_width(self, val: f32) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().border_width = val;
        }
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().shape = shape;
        }
        self
    }

    pub fn with_rotation_deg(self, deg: f32) -> Self {
        self.with_rotation_rad(deg.to_radians())
    }

    pub fn with_rotation_rad(self, rad: f32) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().transform.set_rotate_rad(rad);
        }
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().corner_radius = val;
        }
        self
    }

    pub fn with_horizontal_align(self, align_h: AlignH) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().align_h = align_h;
        }
        self
    }

    pub fn with_vertical_align(self, align_v: AlignV) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().align_v = align_v;
        }
        self
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().orientation = orientation;
        }
        self
    }

    pub fn hoverable(self) -> Self {
        if let Some(state) = self.try_upgrade() {
            state.borrow_mut().flag.set_hoverable(true);
        }
        self
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

    pub fn set_border_width(&self, val: f32) {
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
            node.borrow_mut().image_aspect_ratio = val;
            node.borrow_mut().flag.set_dirty(true);
        }
    }
}
