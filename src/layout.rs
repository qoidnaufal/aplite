use aplite_types::{Rect, Vec2f, Size};
use aplite_storage::{SparseIndices, Array, Tree};

use crate::state::{WidgetState, AspectRatio, Flag};
use crate::widget::{Widget, WidgetId};

pub struct LayoutCx {
    pub(crate) next_pos: Vec2f,
    pub(crate) rules: Rules,
}

impl LayoutCx {
    pub fn new(parent: &dyn Widget) -> Self {
        let node = parent.node_ref().unwrap().upgrade();
        let rules = Rules::new(&node.borrow());

        let(total_size, len) = parent.children_ref()
            .map(|children| {
                (
                    children.iter()
                        .map(|child| {
                            let rect = child.node_ref().unwrap().upgrade().borrow().rect;
                            match rules.orientation {
                                Orientation::Vertical => rect.size().height,
                                Orientation::Horizontal => rect.size().width,
                            }
                        })
                        .sum::<f32>(),
                    children.len() as f32
                )
            }).unwrap_or_default();

        let next_pos = rules.start_pos(total_size, len);

        Self {
            rules,
            next_pos,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Rules {
    pub(crate) rect: Rect,
    pub(crate) orientation: Orientation,
    pub(crate) align_h: AlignH,
    pub(crate) align_v: AlignV,
    pub(crate) padding: Padding,
    pub(crate) spacing: u8,
}

impl Rules {
    pub(crate) fn new(state: &WidgetState) -> Self {
        Self {
            rect: state.rect,
            orientation: state.orientation,
            align_h: state.align_h,
            align_v: state.align_v,
            padding: state.padding,
            spacing: state.spacing,
        }
    }

    fn offset_x(&self) -> f32 {
        let pl = self.padding.left as f32;
        let pr = self.padding.right as f32;

        match self.align_h {
            AlignH::Left => self.rect.x + pl,
            AlignH::Center => {
                self.rect.x + self.rect.width / 2. + pl - pr
            }
            AlignH::Right => self.rect.max_x() - pr
        }
    }

    fn offset_y(&self) -> f32 {
        let pt = self.padding.top as f32;
        let pb = self.padding.bottom as f32;

        match self.align_v {
            AlignV::Top => self.rect.y + pt,
            AlignV::Middle => {
                self.rect.y + self.rect.height / 2. + pt - pb
            }
            AlignV::Bottom => self.rect.max_y() - pb,
        }
    }

    fn start_pos(&self, child_total_size: f32, len: f32) -> Vec2f {
        let offset_x = self.offset_x();
        let offset_y = self.offset_y();
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

impl<T: Widget + Sized + 'static> Layout for T {}

pub(crate) trait Layout: Widget + Sized + 'static {
    fn calculate_layout(&self, cx: &mut LayoutCx) {
        if self.layout(cx) && let Some(children) = self.children_ref() {
            let mut this_cx = LayoutCx::new(self);
            children.iter()
                .for_each(|child| child.calculate_layout(&mut this_cx));
        }
    }

    fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
        let node = self.node_ref().unwrap().upgrade();
        if node.borrow().flag.is_hidden() { return Size::default() }

        let state = node.borrow();
        let padding = state.padding;
        let orientation = state.orientation;
        let spacing = state.spacing as f32;
        let mut size = state.rect.size();

        if let Some(children) = self.children_ref() {
            let mut expand = Size::default();

            children
                .iter()
                .filter(|child| child.node().is_visible())
                .enumerate()
                .for_each(|(i, child)| {
                    let child_size = child.calculate_size(Some(self));
                    let stretch = spacing * i.clamp(0, 1) as f32;

                    match orientation {
                        Orientation::Vertical => {
                            expand.height += child_size.height + stretch;
                            expand.width = expand.width.max(child_size.width + padding.horizontal() as f32);
                        }
                        Orientation::Horizontal => {
                            expand.height = expand.height.max(child_size.height + padding.vertical() as f32);
                            expand.width += child_size.width + stretch;
                        }
                    }
                });

            match orientation {
                Orientation::Vertical => {
                    expand.height += padding.vertical() as f32;
                },
                Orientation::Horizontal => {
                    expand.width += padding.horizontal() as f32;
                },
            }

            size = expand;
        }

        size = size
            .adjust_on_min_constraints(state.min_width, state.min_height)
            .adjust_on_max_constraints(state.max_width, state.max_height);

        let aspect_ratio = match state.image_aspect_ratio {
            AspectRatio::Defined(n, d) => Some((n, d).into()),
            AspectRatio::Source => node.borrow()
                .background_paint
                .aspect_ratio(),
            AspectRatio::Undefined => None,
        };

        if let Some(fraction) = aspect_ratio {
            match parent {
                Some(parent) if parent
                    .node_ref()
                    .unwrap()
                    .upgrade()
                    .borrow()
                    .orientation
                    .is_vertical() => size.adjust_height_with_fraction(fraction),
                _ => size.adjust_width_with_fraction(fraction),
            }
        }

        if state.rect.size() == size { return size }

        drop(state);

        let mut state = node.borrow_mut();
        state.rect.set_size(size);
        state.flag.set_dirty(true);

        size
    }
}

#[allow(unused)]
pub(crate) mod layout_state {
    use super::*;

    pub(crate) struct LayoutRules {
        pub(crate) orientation: Orientation,
        // pub(crate) align_h: AlignH,
        // pub(crate) align_v: AlignV,
        pub(crate) padding: Padding,
        pub(crate) spacing: u8,
    }

    /// Unit to calculate size
    pub enum Unit {
        /// fixed unit
        Fixed(f32),
        FitToChild,
        Grow,
    }

    pub(crate) struct LayoutState {
        pub(crate) ptr: SparseIndices<WidgetId>,

        pub(crate) position: Vec<Vec2f>,
        pub(crate) size: Vec<Size>,
        pub(crate) flag: Vec<Flag>,

        pub(crate) width: Vec<Unit>,
        pub(crate) min_width: Vec<Option<f32>>,
        pub(crate) max_width: Vec<Option<f32>>,

        pub(crate) height: Vec<Unit>,
        pub(crate) min_height: Vec<Option<f32>>,
        pub(crate) max_height: Vec<Option<f32>>,

        pub(crate) rules: Array<WidgetId, LayoutRules>,
    }

    impl LayoutState {
        pub(crate) fn new() -> Self {
            Self {
                ptr: SparseIndices::default(),
                position: Vec::new(),
                size: Vec::new(),
                flag: Vec::new(),
                width: Vec::new(),
                min_width: Vec::new(),
                max_width: Vec::new(),
                height: Vec::new(),
                min_height: Vec::new(),
                max_height: Vec::new(),
                rules: Array::default(),
            }
        }

        pub(crate) fn calculate_layout(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
            self.update_fixed_unit();

            self.calculate_size(tree, start);

            self.update_constraints();

            self.update_growth_unit(tree, start);

            self.calculate_position(tree, start);
        }

        pub(crate) fn update_fixed_unit(&mut self) {
            self.size
                .iter_mut()
                .zip(self.width.iter().zip(&self.height))
                .zip(&self.flag)
                .filter(|(_, flag)| !flag.is_hidden())
                .for_each(|((size, (width, height)), _)| {
                    match width {
                        Unit::Fixed(w) => size.width = *w,
                        Unit::FitToChild | Unit::Grow => {},
                    };
                    match height {
                        Unit::Fixed(h) => size.height = *h,
                        Unit::FitToChild | Unit::Grow => {},
                    }
                });
        }

        pub(crate) fn update_constraints(&mut self) {
            self.size
                .iter_mut()
                .zip(self.min_width.iter().zip(&self.max_width))
                .zip(self.min_height.iter().zip(&self.max_height))
                .zip(&self.flag)
                .filter(|(_, flag)| !flag.is_hidden())
                .for_each(|(((size, (min_w, max_w)), (min_h, max_h)), _)| {
                    *size = size
                        .adjust_on_min_constraints(*min_w, *min_h)
                        .adjust_on_max_constraints(*max_w, *max_h);
                });
        }

        pub(crate) fn update_growth_unit(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
            tree.iter_breadth(start)
                .filter(|id| !self.ptr.with(*id, |index| &self.flag[index]).unwrap().is_hidden())
                .for_each(|id| {
                    if let Some(rules) = self.rules.get(id) {
                        let size = self.ptr.with(id, |index| &self.size[index]).copied().unwrap();

                        let (rem_w, rem_h) = tree.iter_children(id)
                            .map(|child| self.ptr.with(child, |index| &self.size[index]).unwrap())
                            .fold((size.width, size.height), |(w, h), cs| {
                                match rules.orientation {
                                    Orientation::Horizontal => (w - cs.width, h),
                                    Orientation::Vertical => (w, h - cs.height),
                                }
                            });

                        let to_grow_w = tree.iter_children(id)
                            .filter(|child| matches!(self.ptr.with(*child, |index| &self.width[index]).unwrap(), Unit::Grow))
                            .collect::<Vec<_>>();

                        let to_grow_h = tree.iter_children(id)
                            .filter(|child| matches!(self.ptr.with(*child, |index| &self.height[index]).unwrap(), Unit::Grow))
                            .collect::<Vec<_>>();

                        let count_w = to_grow_w.len() as f32;
                        let count_h = to_grow_h.len() as f32;

                        to_grow_w.iter().for_each(|child| {
                            let cs = self.ptr.with(*child, |index| &mut self.size[index]).unwrap();
                            match rules.orientation {
                                Orientation::Horizontal => cs.width += rem_w / count_w,
                                Orientation::Vertical => cs.width = rem_w,
                            }
                        });

                        to_grow_h.iter().for_each(|child| {
                            let cs = self.ptr.with(*child, |index| &mut self.size[index]).unwrap();
                            match rules.orientation {
                                Orientation::Horizontal => cs.height = rem_h,
                                Orientation::Vertical => cs.height += rem_h / count_h,
                            }
                        });
                    }
                });
        }

        pub(crate) fn calculate_size(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
            tree.iter_breadth(start)
                .rev()
                .filter(|id| !self.ptr.with(*id, |index| &self.flag[index]).unwrap().is_hidden())
                .for_each(|id| {
                    let mut size = Size::default();

                    if let Some(rules) = self.rules.get(id) {
                        let orientation = rules.orientation;
                        let spacing = rules.spacing;
                        let padding = rules.padding;

                        tree.iter_children(id).enumerate().for_each(|(n, child)| {
                            let child_size = self.ptr.with(child, |index| &self.size[index]).unwrap();
                            match orientation {
                                Orientation::Horizontal => {
                                    size.width += child_size.width + spacing as f32 * n.clamp(0, 1) as f32;
                                    size.height = size.height.max(child_size.height);
                                },
                                Orientation::Vertical => {
                                    size.height += child_size.height + spacing as f32 * n.clamp(0, 1) as f32;
                                    size.width = size.width.max(child_size.width);
                                },
                            }
                        });

                        size.width += padding.horizontal() as f32;
                        size.height += padding.vertical() as f32;
                    }

                    if let Some(this_size) = self.ptr.with(id, |index| &mut self.size[index]) {
                        *this_size = size;
                    }
                });
        }

        pub(crate) fn calculate_position(&mut self, tree: &Tree<WidgetId>, start: &WidgetId) {
            tree.iter_breadth(start)
                .filter(|id| !self.ptr.with(*id, |index| &self.flag[index]).unwrap().is_hidden())
                .for_each(|id| {
                    if let Some(parent) = tree.get_parent(id)
                        && let Some(rules) = self.rules.get(parent)
                    {
                        let prev_pos_size = tree.get_prev_sibling(id)
                            .and_then(|prev| {
                                let pos = self.ptr.with(prev, |index| &self.position[index]).copied();
                                let size = self.ptr.with(prev, |index| &self.size[index]).copied();
                                pos.zip(size)
                            });

                        let parent_pos = *self.ptr.with(parent, |index| &self.position[index]).unwrap();

                        let pos = self.ptr.with(id, |index| &mut self.position[index]).unwrap();

                        let orientation = rules.orientation;
                        let spacing = rules.spacing;
                        let padding = rules.padding;

                        if let Some((p, s)) = prev_pos_size {
                            match orientation {
                                Orientation::Horizontal => {
                                    pos.x = p.x + s.width + spacing as f32;
                                    pos.y = p.y;
                                },
                                Orientation::Vertical => {
                                    pos.x = p.x;
                                    pos.y = p.y + s.height + spacing as f32;
                                },
                            }
                        } else {
                            pos.x = parent_pos.x + padding.left as f32;
                            pos.y = parent_pos.y + padding.top as f32;
                        }
                    }
                });
        }

        pub(crate) fn update_alignment(&mut self) {}
    }

    pub(crate) fn update_fixed_unit(size: &mut [Size], width: &[Unit], height: &[Unit], flags: &[Flag]) {
        for ((size, (w, h)), flag) in size.iter_mut().zip(width.iter().zip(height)).zip(flags) {
            if !flag.is_hidden() {
                match w {
                    Unit::Fixed(val) => size.width = *val,
                    Unit::FitToChild | Unit::Grow => {},
                };
                match h {
                    Unit::Fixed(val) => size.height = *val,
                    Unit::FitToChild | Unit::Grow => {},
                }
            }
        }
    }

    pub(crate) fn update_constraints(
        size: &mut [Size],
        min_w: &[Option<f32>],
        max_w: &[Option<f32>],
        min_h: &[Option<f32>],
        max_h: &[Option<f32>],
        flags: &[Flag]
    ) {
        for (((size, (min_w, max_w)), (min_h, max_h)), flag) in size.iter_mut()
            .zip(min_w.iter().zip(max_w))
            .zip(min_h.iter().zip(max_h))
            .zip(flags)
        {
            if !flag.is_hidden() {
                *size = size
                    .adjust_on_min_constraints(*min_w, *min_h)
                    .adjust_on_max_constraints(*max_w, *max_h);
            }
        }
    }
}
