use std::collections::HashMap;

use aplite_types::{
    Rect,
    Size,
    Unit
};
use aplite_storage::{
    Query,
    Tree,
};

use crate::state::{State, Flag};
use crate::widget::WidgetId;

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

#[derive(Default)]
pub struct LayoutRules {
    pub(crate) padding: Padding,
    pub(crate) orientation: Orientation,
    pub(crate) align_h: AlignH,
    pub(crate) align_v: AlignV,
    pub(crate) spacing: u8,
}

pub(crate) struct Layout {
    pub(crate) tree: Tree<WidgetId>,
    pub(crate) rules: HashMap<WidgetId, LayoutRules>,
}

impl Layout {
    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            tree: Tree::with_capacity(capacity),
            rules: HashMap::default(),
        }
    }

    pub(crate) fn calculate_layout(&self, start: &WidgetId, state: &State) {
        update_fixed_unit(state.common.query::<(&mut Rect, &(Unit, Unit), &Flag)>());

        self.calculate_size(start, state);

        // self.update_constraints();

        self.update_growth_unit(start, state);

        self.calculate_position(start, state);
    }

    // pub(crate) fn update_constraints(&mut self) {
    //     self.size
    //         .iter_mut()
    //         .zip(self.min_width.iter().zip(&self.max_width))
    //         .zip(self.min_height.iter().zip(&self.max_height))
    //         .zip(&self.flag)
    //         .filter(|(_, flag)| flag.is_visible())
    //         .for_each(|(((size, (min_w, max_w)), (min_h, max_h)), _)| {
    //             *size = size
    //                 .adjust_on_min_constraints(*min_w, *min_h)
    //                 .adjust_on_max_constraints(*max_w, *max_h);
    //         });
    // }

    pub(crate) fn update_growth_unit(&self, start: &WidgetId, state: &State) {
        self.tree.iter_depth(start)
            .filter(|id| state.common.query::<&Flag>().get(*id).is_some_and(|flag| flag.visible))
            .for_each(|id| {
                if let Some(rules) = self.rules.get(id) {
                    let query_rect = state.common.query::<&Rect>();
                    let size = query_rect.get(id).unwrap().size();

                    let (rem_w, rem_h) = self.tree.iter_children(id)
                        .flat_map(|child| query_rect.get(child))
                        .fold((size.width, size.height), |(w, h), cr| {
                            match rules.orientation {
                                Orientation::Horizontal => (w - cr.width, h),
                                Orientation::Vertical => (w, h - cr.height),
                            }
                        });

                    let query_size = state.common.query::<&(Unit, Unit)>();
                    let to_grow_w = self.tree.iter_children(id)
                        .filter(|child| {
                            query_size.get(*child)
                                .is_some_and(|(width, _)| width.is_grow())
                        })
                        .collect::<Vec<_>>();

                    let to_grow_h = self.tree.iter_children(id)
                        .filter(|child| {
                            query_size.get(*child)
                                .is_some_and(|(_, height)| height.is_grow())
                        })
                        .collect::<Vec<_>>();

                    let count_w = to_grow_w.len() as f32;
                    let count_h = to_grow_h.len() as f32;

                    drop(query_rect);

                    let query_rect_mut = state.common.query::<&mut Rect>();

                    to_grow_w.iter().for_each(|child| {
                        let child_rect = query_rect_mut.get(*child).unwrap();
                        match rules.orientation {
                            Orientation::Horizontal => child_rect.width += rem_w / count_w,
                            Orientation::Vertical => child_rect.width = rem_w,
                        }
                    });

                    to_grow_h.iter().for_each(|child| {
                        let child_rect = query_rect_mut.get(*child).unwrap();
                        match rules.orientation {
                            Orientation::Horizontal => child_rect.height = rem_h,
                            Orientation::Vertical => child_rect.height += rem_h / count_h,
                        }
                    });
                }
            });
    }

    // at the same time makes any container fit to child
    fn calculate_size(&self, start: &WidgetId, state: &State) -> Size {
        let mut size = Size::default();

        if state.common
            .query::<&Flag>()
            .get(start)
            .is_some_and(|flag| !flag.visible) { return size }

        if let Some(rules) = self.rules.get(start) {
            let orientation = rules.orientation;
            let padding = rules.padding;
            let spacing = rules.spacing;

            let children = self.tree.iter_children(start).copied().collect::<Vec<_>>();
            let child_count = children
                .iter()
                .map(|child| {
                    let child_size = self.calculate_size(&child, state);
                    match orientation {
                        Orientation::Horizontal => {
                            size.width += child_size.width;
                            size.height = size.height.max(child_size.height);
                        },
                        Orientation::Vertical => {
                            size.height += child_size.height;
                            size.width = size.width.max(child_size.width);
                        },
                    }
                })
                .count();

            match orientation {
                Orientation::Horizontal => size.width += spacing as f32 * (child_count - 1) as f32,
                Orientation::Vertical => size.height += spacing as f32 * (child_count - 1) as f32,
            }

            size.width += padding.horizontal() as f32;
            size.height += padding.vertical() as f32;
        }

        if let Some(this_size) = state.common.query::<&mut Rect>().get(start) {
            this_size.set_size(size);
        }

        size
    }

    pub(crate) fn calculate_position(&self, start: &WidgetId, state: &State) {
        self.tree.iter_depth(start)
            .filter(|id| state.common.query::<&Flag>().get(*id).is_some_and(|flag| flag.visible))
            .for_each(|id| {
                if let Some(parent) = self.tree.get_parent(id)
                    && let Some(rules) = self.rules.get(parent)
                {
                    let query_rect = state.common.query::<&mut Rect>();
                    let prev_rect = self.tree.get_prev_sibling(id)
                        .and_then(|prev| query_rect.get(prev).copied());

                    let parent_pos = query_rect.get(parent).copied().unwrap();
                    let pos = query_rect.get(id).unwrap();

                    let orientation = rules.orientation;
                    let spacing = rules.spacing;
                    let padding = rules.padding;

                    if let Some(rect) = prev_rect {
                        match orientation {
                            Orientation::Horizontal => {
                                pos.x = rect.x + rect.width + spacing as f32;
                                pos.y = rect.y;
                            },
                            Orientation::Vertical => {
                                pos.x = rect.x;
                                pos.y = rect.y + rect.height + spacing as f32;
                            },
                        }
                    } else {
                        pos.x = parent_pos.x + padding.left as f32;
                        pos.y = parent_pos.y + padding.top as f32;
                    }
                }
            });
    }

    // pub(crate) fn update_alignment(&mut self) {}
}

pub(crate) fn update_fixed_unit<'a>(query: Query<'a, (&'a mut Rect, &'a (Unit, Unit), &'a Flag)>) {
    query.iter()
        .filter(|(_, _, flag)| flag.visible)
        .for_each(|(rect, (width, height), _)| {
            rect.width = width.get();
            rect.height = height.get();
        });
}

// pub struct LayoutCx {
//     pub(crate) next_pos: Vec2f,
//     pub(crate) rules: Rules,
// }

// impl LayoutCx {
//     pub fn new(parent: &dyn Widget) -> Self {
//         let node = parent.node_ref().unwrap().upgrade();
//         let rules = Rules::new(&node.borrow());

//         let(total_size, len) = parent.children_ref()
//             .map(|children| {
//                 (
//                     children.iter()
//                         .map(|child| {
//                             let rect = child.node_ref().unwrap().upgrade().borrow().rect;
//                             match rules.orientation {
//                                 Orientation::Vertical => rect.size().height,
//                                 Orientation::Horizontal => rect.size().width,
//                             }
//                         })
//                         .sum::<f32>(),
//                     children.len() as f32
//                 )
//             }).unwrap_or_default();

//         let next_pos = rules.start_pos(total_size, len);

//         Self {
//             rules,
//             next_pos,
//         }
//     }
// }

// #[derive(Debug)]
// pub(crate) struct Rules {
//     pub(crate) rect: Rect,
//     pub(crate) orientation: Orientation,
//     pub(crate) align_h: AlignH,
//     pub(crate) align_v: AlignV,
//     pub(crate) padding: Padding,
//     pub(crate) spacing: u8,
// }

// impl Rules {
//     pub(crate) fn new(state: &WidgetState) -> Self {
//         Self {
//             rect: state.rect,
//             orientation: state.orientation,
//             align_h: state.align_h,
//             align_v: state.align_v,
//             padding: state.padding,
//             spacing: state.spacing,
//         }
//     }

//     fn offset_x(&self) -> f32 {
//         let pl = self.padding.left as f32;
//         let pr = self.padding.right as f32;

//         match self.align_h {
//             AlignH::Left => self.rect.x + pl,
//             AlignH::Center => {
//                 self.rect.x + self.rect.width / 2. + pl - pr
//             }
//             AlignH::Right => self.rect.max_x() - pr
//         }
//     }

//     fn offset_y(&self) -> f32 {
//         let pt = self.padding.top as f32;
//         let pb = self.padding.bottom as f32;

//         match self.align_v {
//             AlignV::Top => self.rect.y + pt,
//             AlignV::Middle => {
//                 self.rect.y + self.rect.height / 2. + pt - pb
//             }
//             AlignV::Bottom => self.rect.max_y() - pb,
//         }
//     }

//     fn start_pos(&self, child_total_size: f32, len: f32) -> Vec2f {
//         let offset_x = self.offset_x();
//         let offset_y = self.offset_y();
//         let stretch_factor = self.spacing as f32 * (len - 1.);
//         let stretch = child_total_size + stretch_factor;

//         match self.orientation {
//             Orientation::Vertical => {
//                 let y = match self.align_v {
//                     AlignV::Top => offset_y,
//                     AlignV::Middle => offset_y - stretch / 2.,
//                     AlignV::Bottom => offset_y - stretch,
//                 };
//                 Vec2f::new(offset_x, y)
//             },
//             Orientation::Horizontal => {
//                 let x = match self.align_h {
//                     AlignH::Left => offset_x,
//                     AlignH::Center => offset_x - stretch / 2.,
//                     AlignH::Right => offset_x - stretch,
//                 };
//                 Vec2f::new(x, offset_y)
//             }
//         }
//     }
// }

// impl<T: Widget + Sized + 'static> Layout for T {}

// pub(crate) trait Layout: Widget + Sized + 'static {
//     fn calculate_layout(&self, cx: &mut LayoutCx) {
//         if self.layout(cx) && let Some(children) = self.children_ref() {
//             let mut this_cx = LayoutCx::new(self);
//             children.iter()
//                 .for_each(|child| child.calculate_layout(&mut this_cx));
//         }
//     }

//     fn calculate_size(&self, parent: Option<&dyn Widget>) -> Size {
//         let node = self.node_ref().unwrap().upgrade();
//         if node.borrow().flag.is_hidden() { return Size::default() }

//         let state = node.borrow();
//         let padding = state.padding;
//         let orientation = state.orientation;
//         let spacing = state.spacing as f32;
//         let mut size = state.rect.size();

//         if let Some(children) = self.children_ref() {
//             let mut expand = Size::default();

//             children
//                 .iter()
//                 .filter(|child| child.node().is_visible())
//                 .enumerate()
//                 .for_each(|(i, child)| {
//                     let child_size = child.calculate_size(Some(self));
//                     let stretch = spacing * i.clamp(0, 1) as f32;

//                     match orientation {
//                         Orientation::Vertical => {
//                             expand.height += child_size.height + stretch;
//                             expand.width = expand.width.max(child_size.width + padding.horizontal() as f32);
//                         }
//                         Orientation::Horizontal => {
//                             expand.height = expand.height.max(child_size.height + padding.vertical() as f32);
//                             expand.width += child_size.width + stretch;
//                         }
//                     }
//                 });

//             match orientation {
//                 Orientation::Vertical => {
//                     expand.height += padding.vertical() as f32;
//                 },
//                 Orientation::Horizontal => {
//                     expand.width += padding.horizontal() as f32;
//                 },
//             }

//             size = expand;
//         }

//         size = size
//             .adjust_on_min_constraints(state.min_width, state.min_height)
//             .adjust_on_max_constraints(state.max_width, state.max_height);

//         let aspect_ratio = match state.image_aspect_ratio {
//             AspectRatio::Defined(n, d) => Some((n, d).into()),
//             AspectRatio::Source => node.borrow()
//                 .background_paint
//                 .aspect_ratio(),
//             AspectRatio::Undefined => None,
//         };

//         if let Some(fraction) = aspect_ratio {
//             match parent {
//                 Some(parent) if parent
//                     .node_ref()
//                     .unwrap()
//                     .upgrade()
//                     .borrow()
//                     .orientation
//                     .is_vertical() => size.adjust_height_with_fraction(fraction),
//                 _ => size.adjust_width_with_fraction(fraction),
//             }
//         }

//         if state.rect.size() == size { return size }

//         drop(state);

//         let mut state = node.borrow_mut();
//         state.rect.set_size(size);
//         state.flag.set_dirty(true);

//         size
//     }
// }
