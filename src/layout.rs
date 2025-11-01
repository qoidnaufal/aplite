use aplite_types::{
    Rect,
    Vec2f,
    Size,
    Unit
};
use aplite_storage::{
    EntityId,
    Array,
    Tree,
};

use crate::state::{Flag, AspectRatio};
use crate::view::IntoView;
use crate::widget::Widget;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignH {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignV {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Padding {
    pub top: u8,
    pub bottom: u8,
    pub left: u8,
    pub right: u8,
}

impl Orientation {
    pub fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl Padding {
    pub const fn new(top: u8, bottom: u8, left: u8, right: u8) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub const fn splat(value: u8) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub(crate) fn vertical(&self) -> u8 { self.top + self.bottom }

    pub(crate) fn horizontal(&self) -> u8 { self.left + self.right }

    pub fn set_all(&mut self, value: u8) {
        self.top = value;
        self.bottom = value;
        self.left = value;
        self.right = value;
    }
}

pub struct LayoutCx<'a> {
    pub(crate) next_pos: Vec2f,
    pub(crate) rules: &'a LayoutRules,
}

impl<'a> LayoutCx<'a> {
    pub fn new(rules: &'a LayoutRules, rect: &'a Rect, child_size: f32, child_count: usize) -> Self {
        let next_pos = rules.start_pos(rect, child_size, child_count as f32);
        Self {
            rules,
            next_pos,
        }
    }
}

#[derive(Default)]
pub struct LayoutRules {
    pub(crate) padding: Padding,
    pub(crate) orientation: Orientation,
    pub(crate) align_h: AlignH,
    pub(crate) align_v: AlignV,
    pub(crate) spacing: u8,
}

impl LayoutRules {
    fn offset_x(&self, rect: &Rect) -> f32 {
        let pl = self.padding.left as f32;
        let pr = self.padding.right as f32;

        match self.align_h {
            AlignH::Left => rect.x + pl,
            AlignH::Center => {
                rect.x + rect.width / 2. + pl - pr
            }
            AlignH::Right => rect.max_x() - pr
        }
    }

    fn offset_y(&self, rect: &Rect) -> f32 {
        let pt = self.padding.top as f32;
        let pb = self.padding.bottom as f32;

        match self.align_v {
            AlignV::Top => rect.y + pt,
            AlignV::Middle => {
                rect.y + rect.height / 2. + pt - pb
            }
            AlignV::Bottom => rect.max_y() - pb,
        }
    }

    fn start_pos(&self, rect: &Rect, child_total_size: f32, len: f32) -> Vec2f {
        let offset_x = self.offset_x(rect);
        let offset_y = self.offset_y(rect);
        let stretch_factor = self.spacing as f32 * (len - 1.);
        let stretch = child_total_size + stretch_factor;

        match self.orientation {
            Orientation::Vertical => {
                let y = match self.align_v {
                    AlignV::Top => offset_y,
                    AlignV::Middle => offset_y - stretch / 2.,
                    AlignV::Bottom => offset_y - stretch,
                };
                Vec2f::new(offset_x, y)
            },
            Orientation::Horizontal => {
                let x = match self.align_h {
                    AlignH::Left => offset_x,
                    AlignH::Center => offset_x - stretch / 2.,
                    AlignH::Right => offset_x - stretch,
                };
                Vec2f::new(x, offset_y)
            }
        }
    }
}

pub struct LayoutNode {
    pub(crate) width: Unit,
    pub(crate) height: Unit,
    pub(crate) min_width: Option<f32>,
    pub(crate) min_height: Option<f32>,
    pub(crate) max_width: Option<f32>,
    pub(crate) max_height: Option<f32>,
}

impl LayoutNode {
    pub fn from_radius(val: Unit) -> Self {
        todo!()
    }
}

pub struct Layout {
    pub(crate) window_rect: Rect,
    pub(crate) tree: Tree,
    pub(crate) rects: Array<Rect>,
}

impl Layout {
    pub(crate) fn new(window_rect: Rect) -> Self {
        Self::with_capacity(window_rect, 0)
    }

    pub(crate) fn with_capacity(window_rect: Rect, capacity: usize) -> Self {
        Self {
            window_rect,
            tree: Tree::with_capacity(capacity),
            rects: Array::with_capacity(capacity),
        }
    }

    pub(crate) fn get_or_insert(&mut self, id: EntityId, parent: Option<EntityId>, rect: impl FnOnce() -> Rect) -> &mut Rect {
        self.tree.insert(id, parent);
        self.rects.get_or_insert(&id, rect)
    }

    pub(crate) fn insert(&mut self, id: EntityId, parent: Option<EntityId>, rect: Rect) {
        self.rects.insert(&id, rect);
        self.tree.insert(id, parent);
    }

    pub(crate) fn parent_rect(&self, id: &EntityId) -> Option<&Rect> {
        self.tree
            .get_parent(id)
            .and_then(|parent| self.rects.get(parent))
    }
}

pub(crate) trait LayoutTrait: Widget + 'static {
    fn calculate_layout(&mut self, cx: &mut LayoutCx) {
        todo!()
    }

    fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
        todo!()
    }
}
