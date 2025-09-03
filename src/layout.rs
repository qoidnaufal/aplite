use std::collections::HashMap;

use aplite_types::{Rect, Vec2f, Size};
use aplite_storage::{DataPointer, DataStore, Tree};

use crate::state::{WidgetState, AspectRatio};
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

/// Unit to calculate size
pub enum Unit {
    /// fixed unit
    Fixed(f32),
    FitToChild,
    Grow,
}

// this will produce a Rect
pub(crate) struct LayoutState {
    ptr: DataPointer<WidgetId>,
    pub(crate) position: Vec<Vec2f>,
    pub(crate) size: Vec<Size>,

    pub(crate) width: Vec<Unit>,
    pub(crate) min_width: Vec<Option<f32>>,
    pub(crate) max_width: Vec<Option<f32>>,

    pub(crate) height: Vec<Unit>,
    pub(crate) min_height: Vec<Option<f32>>,
    pub(crate) max_height: Vec<Option<f32>>,

    pub(crate) rules: DataStore<WidgetId, Rules>,
}

impl LayoutState {
    // there's no recursion here, but there's unnecessary(?) allocation on the iterator
    pub(crate) fn calculate_size_breadth(&mut self, tree: &Tree<WidgetId>, start: WidgetId) {
        self.update_fixed_unit();

        tree.iter_breadth(start).rev().for_each(|id| {
            let mut size = Size::default();

            if let Some(rules) = self.rules.get(id) {
                let orientation = rules.orientation;
                let spacing = rules.spacing;
                let padding = rules.padding;

                tree.iter_children(id).enumerate().for_each(|(n, child)| {
                    let child_size = self.ptr.get(child, &self.size).unwrap();
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

            if let Some((min_width, max_width, min_height, max_height)) = self.ptr.with(start, |index| {
                (
                    &self.min_width[index],
                    &self.max_width[index],
                    &self.min_height[index],
                    &self.max_width[index],
                )
            }) {
                size = size
                    .adjust_on_min_constraints(*min_width, *min_height)
                    .adjust_on_max_constraints(*max_width, *max_height);
            }

            if let Some(this_size) = self.ptr.get_mut(id, &mut self.size) {
                *this_size = size;
            }
        });
    }

    // using depth first has no extra allocation on the iterator but at the cost of recursive function
    pub(crate) fn calculate_size_depth(&mut self, tree: &Tree<WidgetId>, start: WidgetId) -> Size {
        let mut size = Size::default();

        tree.iter_depth(start).for_each(|id| {
            if let Some(rules) = self.rules.get(id) {
                let orientation = rules.orientation;
                let padding = rules.padding;
                let spacing = rules.spacing as f32;

                tree.iter_children(id).enumerate().for_each(|(n, child)| {
                    let child_size = self.calculate_size_depth(tree, child);
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
        });

        if let Some((min_width, max_width, min_height, max_height)) = self.ptr.with(start, |index| {
            (
                &self.min_width[index],
                &self.max_width[index],
                &self.min_height[index],
                &self.max_width[index],
            )
        }) {
            size = size
                .adjust_on_min_constraints(*min_width, *min_height)
                .adjust_on_max_constraints(*max_width, *max_height);
        }

        // do some extra calculations here if needed

        if let Some(this_size) = self.ptr.with(start, |index| &mut self.size[index]) {
            *this_size = size;
        }

        size
    }

    pub(crate) fn calculate_growth(&self) {}

    pub(crate) fn calculate_position(&self) {}

    pub(crate) fn update_fixed_unit(&mut self) {
        self.size
            .iter_mut()
            .zip(&self.width)
            .zip(&self.height)
            .for_each(|((size, width), height)| {
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
}
