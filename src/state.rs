use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use aplite_renderer::Shape;
use aplite_storage::IndexMap;
use aplite_types::{
    Matrix3x2,
    Rect,
    Size,
    CornerRadius,
    Paint,
    Rgba,
};

use crate::layout::{AlignV, AlignH, Orientation, Padding};
use crate::widget::WidgetId;

thread_local! {
    pub(crate) static NODE_STORAGE: RefCell<IndexMap<WidgetId, Rc<RefCell<WidgetState>>>> =
        RefCell::new(IndexMap::with_capacity(1024));
}

const VERSION_MASK: u8 = 0xFF;
const INDEX_BITS: u8 = 24;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;
const MINIMUM_FREE_INDEX: usize = 1024;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u32);

impl EntityId {
    fn new(id: u32, version: u8) -> Self {
        Self((version as u32) << INDEX_BITS | id)
    }

    pub(crate) fn index(&self) -> usize {
        (self.0 & INDEX_MASK) as usize
    }

    pub(crate) fn version(&self) -> u8 {
        ((self.0 >> INDEX_BITS) as u8) & VERSION_MASK
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.0)
    }
}

struct Manager {
    tree: IndexMap<WidgetId, Tree>
}

struct Tree {
    parent: Option<WidgetId>,
    children: Vec<WidgetId>,
}

#[derive(Debug)]
pub(crate) struct EntityManager {
    recycled: VecDeque<u32>,
    version_manager: Vec<u8>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            recycled: VecDeque::with_capacity(MINIMUM_FREE_INDEX),
            version_manager: Vec::with_capacity(MINIMUM_FREE_INDEX),
        }
    }
}

impl EntityManager {
    pub(crate) fn create(&mut self) -> EntityId {
        let id = if self.recycled.len() > MINIMUM_FREE_INDEX {
            self.recycled.pop_front().unwrap()
        } else {
            self.version_manager.push(0);
            let id = (self.version_manager.len() - 1) as u32;
            assert!(id < (1 << INDEX_BITS));
            id
        };
        EntityId::new(id, self.version_manager[id as usize])
    }

    fn alive(&self, id: &EntityId) -> bool {
        self.version_manager[id.index()] == id.version()
    }

    fn destroy(&mut self, id: EntityId) {
        let idx = id.index();
        self.version_manager[idx] += 1;
        self.recycled.push_back(idx as u32);
    }
}

pub(crate) struct StateManager {
    pub(crate) common: HashMap<EntityId, usize>,
    pub(crate) layout: HashMap<EntityId, usize>,
    pub(crate) paint: HashMap<EntityId, usize>,
    pub(crate) border: HashMap<EntityId, usize>,
}

pub(crate) struct CommonState {
    pub(crate) rect: Vec<Rect>,
    pub(crate) transform: Vec<Matrix3x2>,
    pub(crate) flag: Vec<Flag>,
    pub(crate) shape: Vec<Shape>,
    pub(crate) corner_radius: Vec<CornerRadius>,
    // pub(crate) rotation: f32, // in radians
}

// I think it's okay not to pack this into vec since this will be used rarely
pub(crate) struct SizeConstraint {
    pub(crate) min_width: Vec<Option<f32>>,
    pub(crate) min_height: Vec<Option<f32>>,
    pub(crate) max_width: Vec<Option<f32>>,
    pub(crate) max_height: Vec<Option<f32>>,
}

// I think it's okay not to pack this into vec since this will be used rarely
pub(crate) struct LayoutRules {
    pub(crate) padding: Padding,
    pub(crate) align_v: AlignV,
    pub(crate) align_h: AlignH,
    pub(crate) orientation: Orientation,
    pub(crate) spacing: u8,
}

pub(crate) struct PaintState {
    pub(crate) background_paint: Vec<Paint>,
    pub(crate) aspect_ratio: Vec<AspectRatio>,
}

pub(crate) struct BorderState {
    pub(crate) border_paint: Vec<Rgba>,
    pub(crate) border_width: Vec<u8>,
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
