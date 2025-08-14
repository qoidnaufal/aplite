use aplite_types::{Rect, Vec2f};

use crate::state::{AspectRatio, WidgetState};
use crate::view::{ViewNodeRef, ViewNode};
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
    Vertical,
    Horizontal,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Debug)]
pub(crate) struct Rules {
    pub(crate) rect: Rect,
    pub(crate) orientation: Orientation,
    pub(crate) align_h: AlignH,
    pub(crate) align_v: AlignV,
    pub(crate) padding: Padding,
    pub(crate) spacing: f32,
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
    pub const fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub const fn splat(value: f32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub(crate) fn vertical(&self) -> f32 { self.top + self.bottom }

    pub(crate) fn horizontal(&self) -> f32 { self.left + self.right }

    pub fn set_all(&mut self, value: f32) {
        self.top = value;
        self.bottom = value;
        self.left = value;
        self.right = value;
    }
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
        let pl = self.padding.left;
        let pr = self.padding.right;

        match self.align_h {
            AlignH::Left => self.rect.x + pl,
            AlignH::Center => {
                self.rect.x + self.rect.width / 2. + pl - pr
            }
            AlignH::Right => self.rect.max_x() - pr
        }
    }

    fn offset_y(&self) -> f32 {
        let pt = self.padding.top;
        let pb = self.padding.bottom;

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
        let stretch_factor = self.spacing * (len - 1.);
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

pub struct LayoutCx {
    pub(crate) next_pos: Vec2f,
    pub(crate) rules: Rules,
}

impl LayoutCx {
    pub fn new(parent: &mut dyn Widget) -> Self {
        let node = parent.node();
        let rules = Rules::new(&node.borrow());

        let(total_size, len) = parent.children_ref()
            .map(|children| {
                (
                    children.iter()
                        .map(|child| {
                            let rect = child.node().borrow().rect;
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

    // pub(crate) fn calculate(&mut self) {
    //     let children = VIEW_STORAGE.with(|s| {
    //         s.tree
    //             .borrow()
    //             .get_all_children(&self.entity)
    //     });

    //     self.initialize_next_pos(children.as_ref());

    //     if let Some(children) = children {
    //         children.iter().for_each(|child| {
    //             self.assign_position(child);
    //             Self::new(*child).calculate();
    //         });
    //     }
    // }

    // fn initialize_next_pos(&mut self, children: Option<&Vec<ViewId>>) {
    //     let (child_total_size, len) = children.map(|c| {(
    //         c.iter()
    //             .map(|child| {
    //                 VIEW_STORAGE.with(|s| {
    //                     let rect = s.get_widget_state(child).unwrap().borrow().rect;

    //                     match self.rules.orientation {
    //                         Orientation::Vertical => rect.size().height,
    //                         Orientation::Horizontal => rect.size().width,
    //                     }
    //                 })
    //             })
    //             .sum(),
    //         c.len() as f32
    //     )})
    //     .unwrap_or_default();

    //     self.next_pos = self.rules.start_pos(child_total_size, len);
    // }

    // // FIXME: this is not possible without signal?
    // fn assign_position(&mut self, child: &ViewId) {
    //     VIEW_STORAGE.with(|s| {
    //         let state = s.get_widget_state(child).unwrap();
    //         let size = state.borrow().rect.size();
    //         let mut state = state.borrow_mut();

    //         match self.rules.orientation {
    //             Orientation::Vertical => {
    //                 match self.rules.align_h {
    //                     AlignH::Left | AlignH::Right => state.rect.x = self.next_pos.x,
    //                     AlignH::Center => state.rect.x = self.next_pos.x - size.width / 2.,
    //                 }
    //                 state.rect.y = self.next_pos.y;
    //                 self.next_pos.y += self.rules.spacing + size.height;
    //             },
    //             Orientation::Horizontal => {
    //                 state.rect.x = self.next_pos.x;
    //                 match self.rules.align_v {
    //                     AlignV::Top | AlignV::Bottom => state.rect.y = self.next_pos.y,
    //                     AlignV::Middle => state.rect.y = self.next_pos.y - size.height / 2.,
    //                 }
    //                 self.next_pos.x += self.rules.spacing + size.width;
    //             },
    //         }
    //     });
    // }

// pub(crate) fn calculate_size_recursive(id: &ViewId) -> Size {
//     VIEW_STORAGE.with(|s| {
//         let tree = s.tree.borrow();
//         let state = tree.get(id).unwrap().borrow();
//         let padding = state.padding;
//         let orientation = state.orientation;
//         let spacing = state.spacing;
//         let mut size = state.rect.size();
//         let maybe_children = tree.get_all_children(id);

//         drop(state);
//         drop(tree);

//         if let Some(children) = maybe_children {
//             children.iter().for_each(|child_id| {
//                 let child_size = calculate_size_recursive(child_id);
//                 match orientation {
//                     Orientation::Vertical => {
//                         size.height += child_size.height;
//                         size.width = size.width.max(child_size.width + padding.horizontal());
//                     }
//                     Orientation::Horizontal => {
//                         size.height = size.height.max(child_size.height + padding.vertical());
//                         size.width += child_size.width;
//                     }
//                 }
//             });
//             let child_len = children.len() as f32;
//             let stretch = spacing * (child_len - 1.);
//             match orientation {
//                 Orientation::Vertical => {
//                     size.height += padding.vertical() + stretch;
//                 },
//                 Orientation::Horizontal => {
//                     size.width += padding.horizontal() + stretch;
//                 },
//             }
//         }

//         let tree = s.tree.borrow();
//         let state = tree.get(id).unwrap().borrow();

//         if let AspectRatio::Defined(tuple) = state.image_aspect_ratio {
//             match tree.get_parent(id) {
//                 Some(parent) if tree
//                     .get(parent)
//                     .unwrap()
//                     .borrow()
//                     .orientation
//                     .is_vertical() => size.adjust_height_aspect_ratio(tuple.into()),
//                 _ => size.adjust_width_aspect_ratio(tuple.into()),
//             }
//         }

//         let final_size = size
//             .adjust_on_min_constraints(state.min_width, state.min_height)
//             .adjust_on_max_constraints(state.max_width, state.max_height);

//         drop(state);
//         drop(tree);

//         let mut tree = s.tree.borrow_mut();
//         let mut state = tree.get_mut(id).unwrap().borrow_mut();
//         state.rect.set_size(final_size);

//         final_size
//     })
// }
