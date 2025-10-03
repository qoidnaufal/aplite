use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::{Renderer, DrawArgs, Shape};
use aplite_storage::{SparseIndices, Tree, Entity};
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
use crate::layout::Layout;

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

    pub(crate) border_paint: Paint,
    pub(crate) border_width: u32,
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

pub(crate) struct State {
    pub(crate) ptr: SparseIndices<WidgetId>,
    pub(crate) entities: Vec<WidgetId>,

    pub(crate) transform: Vec<Matrix3x2>,
    pub(crate) width: Vec<Unit>,
    pub(crate) height: Vec<Unit>,

    // pub(crate) min_width: Vec<Option<f32>>,
    // pub(crate) max_width: Vec<Option<f32>>,

    // pub(crate) min_height: Vec<Option<f32>>,
    // pub(crate) max_height: Vec<Option<f32>>,

    pub(crate) flag: Vec<Flag>,
    pub(crate) shape: Vec<Shape>,
    pub(crate) corner_radius: Vec<CornerRadius>,
    pub(crate) background: Vec<Paint>,
    pub(crate) border_paint: Vec<Paint>,
    pub(crate) border_width: Vec<u32>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            ptr: SparseIndices::default(),
            entities: Vec::with_capacity(capacity),

            transform: Vec::with_capacity(capacity),
            width: Vec::with_capacity(capacity),
            height: Vec::with_capacity(capacity),

            // min_width: Vec::with_capacity(capacity),
            // max_width: Vec::with_capacity(capacity),
            // min_height: Vec::with_capacity(capacity),
            // max_height: Vec::with_capacity(capacity),

            flag: Vec::with_capacity(capacity),
            shape: Vec::with_capacity(capacity),
            corner_radius: Vec::with_capacity(capacity),
            background: Vec::with_capacity(capacity),
            border_paint: Vec::with_capacity(capacity),
            border_width: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn insert_state(&mut self, id: &WidgetId, state: &WidgetState) {
        let WidgetState {
            width,
            height,
            transform,
            shape,
            corner_radius,
            flag,
            background_paint,
            border_paint,
            border_width,
            ..
        } = state.clone();

        self.ptr.resize_if_needed(id);
        self.entities.push(*id);

        self.transform.push(transform);
        self.width.push(width);
        self.height.push(height);

        self.flag.push(flag);
        self.shape.push(shape);
        self.corner_radius.push(corner_radius);

        self.background.push(background_paint);
        self.border_paint.push(border_paint);
        self.border_width.push(border_width);
    }

    pub(crate) fn insert_default_state(&mut self, id: &WidgetId) {
        self.ptr.resize_if_needed(id);
        self.entities.push(*id);

        self.transform.push(Matrix3x2::identity());
        self.width.push(Default::default());
        self.height.push(Default::default());

        self.flag.push(Default::default());
        self.background.push(Paint::Color(Rgba::RED));
        self.shape.push(Shape::RoundedRect);
        self.corner_radius.push(CornerRadius::default());
    }

    pub(crate) fn get_flag(&self, id: &WidgetId) -> Option<&Flag> {
        self.ptr.with(id, |index| self.flag.get(index))?
    }

    pub(crate) fn get_flag_mut(&mut self, id: &WidgetId) -> Option<&mut Flag> {
        self.ptr.with(id, |index| self.flag.get_mut(index))?
    }

    pub(crate) fn get_transform(&self, id: &WidgetId) -> Option<&Matrix3x2> {
        self.ptr.with(id, |index| self.transform.get(index))?
    }

    pub(crate) fn set_width(&mut self, id: &WidgetId, width: Unit) {
        if let Some(w) = self.ptr.with(id, |index| &mut self.width[index]) {
            *w = width
        }
    }

    pub(crate) fn set_height(&mut self, id: &WidgetId, height: Unit) {
        if let Some(h) = self.ptr.with(id, |index| &mut self.height[index]) {
            *h = height
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn remove(&mut self, id: &WidgetId) {
        if self.len() == 0 { return }

        if let Some(index) = self.ptr.get_data_index(id) {
            let last = self.entities.last().unwrap();

            self.ptr.set_index(last.index(), index);
            self.ptr.set_null(id);
            self.entities.swap_remove(index);

            self.width.swap_remove(index);
            self.height.swap_remove(index);

            // self.min_width.swap_remove(index);
            // self.max_width.swap_remove(index);
            // self.min_height.swap_remove(index);
            // self.max_height.swap_remove(index);

            self.flag.swap_remove(index);
            self.background.swap_remove(index);
            self.shape.swap_remove(index);
            self.corner_radius.swap_remove(index);
        }
    }

    pub(crate) fn set_visible(&mut self, tree: &Tree<WidgetId>, id: &WidgetId, visible: bool) {
        tree.iter_depth(id)
            .for_each(|member| {
                if let Some(flag) = self.ptr.with(member, |index| &mut self.flag[index]) {
                    flag.set_hidden(!visible);
                }
            });
    }

    pub(crate) fn render(&self, renderer: &mut Renderer, layout: &Layout) {
        let mut scene = renderer.scene();
        layout.rects.iter()
            .zip(&self.transform)
            .zip(&self.background)
            .zip(&self.border_paint)
            .zip(&self.border_width)
            .zip(&self.corner_radius)
            .zip(&self.shape)
            .zip(&self.flag)
            .for_each(|((((((((_, rect), transform), background), border_paint), border_width), corner_radius), shape), flag)| {
                if flag.is_visible() {
                    let draw_args = DrawArgs {
                        rect,
                        transform,
                        background_paint: &background.as_paint_ref(),
                        border_paint: &border_paint.as_paint_ref(),
                        border_width,
                        shape,
                        corner_radius,
                    };
                    scene.draw(draw_args);
                }
            });
    }
}
