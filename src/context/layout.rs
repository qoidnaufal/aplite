use aplite_reactive::*;
use aplite_types::{Rect, Size, Vec2f};

use crate::widget_state::{AspectRatio, WidgetState};
use crate::view::{ViewId, VIEW_STORAGE};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignH {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignV {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub(crate) h_align: AlignH,
    pub(crate) v_align: AlignV,
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
    rect: Rect,
    orientation: Orientation,
    alignment: Alignment,
    padding: Padding,
    spacing: f32,
}

pub(crate) struct LayoutContext {
    entity: ViewId,
    next_pos: Vec2f,
    rules: Rules,
}

impl Alignment {
    pub const fn new() -> Self {
        Self {
            h_align: AlignH::Center,
            v_align: AlignV::Middle,
        }
    }

    pub fn set_h(&mut self, h_align: AlignH) {
        self.h_align = h_align;
    }

    pub fn set_v(&mut self, v_align: AlignV) {
        self.v_align = v_align;
    }
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
            rect: state.rect.get_untracked(),
            orientation: state.orientation(),
            alignment: state.alignment(),
            padding: state.padding,
            spacing: state.spacing,
        }
    }

    fn offset_x(&self) -> f32 {
        let pl = self.padding.left;
        let pr = self.padding.right;

        match self.alignment.h_align {
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

        match self.alignment.v_align {
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
                let y = match self.alignment.v_align {
                    AlignV::Top => offset_y,
                    AlignV::Middle => offset_y - stretch / 2.,
                    AlignV::Bottom => offset_y - stretch,
                };
                Vec2f::new(offset_x, y)
            },
            Orientation::Horizontal => {
                let x = match self.alignment.h_align {
                    AlignH::Left => offset_x,
                    AlignH::Center => offset_x - stretch / 2.,
                    AlignH::Right => offset_x - stretch,
                };
                Vec2f::new(x, offset_y)
            }
        }
    }
}

impl LayoutContext {
    pub(crate) fn new(entity: ViewId) -> Self {
        let rules = VIEW_STORAGE.with(|s| {
            let widget_state = s.get_widget_state(&entity);
            Rules::new(&widget_state)
        });
        Self {
            entity,
            next_pos: Vec2f::new(0., 0.),
            rules,
        }
    }

    pub(crate) fn calculate(&mut self) {
        let children = VIEW_STORAGE.with(|s| {
            s.tree
                .borrow()
                .get_all_children(&self.entity)
        });

        self.initialize_next_pos(children.as_ref());

        if let Some(children) = children {
            children.iter().for_each(|child| {
                self.assign_position(child);
                Self::new(*child).calculate();
            });
        }
    }

    fn initialize_next_pos(&mut self, children: Option<&Vec<ViewId>>) {
        let (child_total_size, len) = children.map(|c| {(
            c.iter()
                .map(|child| {
                    let rect = VIEW_STORAGE.with(|s| s.get_widget_state(child).rect);
                    rect.read_untracked(|rect| {
                        match self.rules.orientation {
                            Orientation::Vertical => rect.size().height,
                            Orientation::Horizontal => rect.size().width,
                        }
                    })
                })
                .sum(),
            c.len() as f32
        )})
        .unwrap_or_default();

        self.next_pos = self.rules.start_pos(child_total_size, len);
    }

    fn assign_position(&mut self, child: &ViewId) {
        let rect = VIEW_STORAGE.with(|s| s.get_widget_state(child).rect);
        let size = rect.read_untracked(|rect| rect.size());

        match self.rules.orientation {
            Orientation::Vertical => {
                rect.update_untracked(|rect| {
                    match self.rules.alignment.h_align {
                        AlignH::Left | AlignH::Right => rect.x = self.next_pos.x,
                        AlignH::Center => rect.x = self.next_pos.x - size.width / 2.,
                    }
                    rect.y = self.next_pos.y;
                    self.next_pos.y += self.rules.spacing + size.height;
                });
            },
            Orientation::Horizontal => {
                rect.update_untracked(|rect| {
                    rect.x = self.next_pos.x;
                    match self.rules.alignment.v_align {
                        AlignV::Top | AlignV::Bottom => rect.y = self.next_pos.y,
                        AlignV::Middle => rect.y = self.next_pos.y - size.height / 2.,
                    }
                });
                self.next_pos.x += self.rules.spacing + size.width;
            },
        }
    }
}

pub(crate) fn calculate_size_recursive(id: &ViewId) -> Size {
    let widget_state = VIEW_STORAGE.with(|s| s.get_widget_state(id));
    let padding = widget_state.padding;
    let mut size = widget_state.rect.read_untracked(|rect| rect.size());

    let maybe_children = VIEW_STORAGE.with(|s| s.tree.borrow().get_all_children(id));
    if let Some(children) = maybe_children {
        children.iter().for_each(|child_id| {
            let child_size = calculate_size_recursive(child_id);
            match widget_state.orientation() {
                Orientation::Vertical => {
                    size.height += child_size.height;
                    size.width = size.width.max(child_size.width + padding.horizontal());
                }
                Orientation::Horizontal => {
                    size.height = size.height.max(child_size.height + padding.vertical());
                    size.width += child_size.width;
                }
            }
        });
        let child_len = children.len() as f32;
        let stretch = widget_state.spacing * (child_len - 1.);
        match widget_state.orientation() {
            Orientation::Vertical => {
                size.height += padding.vertical() + stretch;
            },
            Orientation::Horizontal => {
                size.width += padding.horizontal() + stretch;
            },
        }
    }

    if let AspectRatio::Defined(tuple) = widget_state.image_aspect_ratio() {
        VIEW_STORAGE.with(|s| {
            match s.tree.borrow().get_parent(id) {
                Some(parent) if s
                    .get_widget_state(parent)
                    .orientation
                    .is_vertical() => size.adjust_height_aspect_ratio(tuple.into()),
                _ => size.adjust_width_aspect_ratio(tuple.into()),
            }
        });
    }

    let final_size = size
        .adjust_on_min_constraints(widget_state.min_width, widget_state.min_height)
        .adjust_on_max_constraints(widget_state.max_width, widget_state.max_height);

    widget_state.rect.update_untracked(|state| state.set_size(final_size));

    final_size
}

// #[derive(Debug)]
// pub(crate) struct Rules {
//     rect: Rect<u32>,
//     orientation: Orientation,
//     alignment: Alignment,
//     padding: Padding,
//     spacing: u32,
// }

// impl Rules {
//     pub(crate) fn new(state: &WidgetState) -> Self {
//         Self {
//             rect: state.rect.get_untracked(),
//             orientation: state.orientation(),
//             alignment: state.alignment(),
//             padding: state.padding(),
//             spacing: state.spacing(),
//         }
//     }

//     fn offset_x(&self) -> u32 {
//         let pl = self.padding.left();
//         let pr = self.padding.right();

//         match self.alignment.h_align {
//             AlignH::Left => self.rect.x() - (self.rect.width() / 2) + pl,
//             AlignH::Center => {
//                 if pl >= pr {
//                     self.rect.x() + (pl - pr) / 2
//                 } else {
//                     self.rect.x() - (pr - pl) / 2
//                 }
//             }
//             AlignH::Right => self.rect.x() + (self.rect.width() / 2) - pr
//         }
//     }

//     fn offset_y(&self) -> u32 {
//         let pt = self.padding.top();
//         let pb = self.padding.bottom();

//         match self.alignment.v_align {
//             AlignV::Top => self.rect.y() - (self.rect.height() / 2) + pt,
//             AlignV::Middle => {
//                 if pt >= pb {
//                     self.rect.y() + (pt - pb) / 2
//                 } else {
//                     self.rect.y() - (pb - pt) / 2
//                 }
//             }
//             AlignV::Bottom => self.rect.y() + (self.rect.height() / 2) - pb,
//         }
//     }

//     fn start_x(&self, total_width: u32, len: u32) -> u32 {
//         let offset_x = self.offset_x();
//         let stretch = self.spacing * (len - 1);
//         let final_width = total_width + stretch;

//         match self.orientation {
//             Orientation::Vertical => offset_x,
//             Orientation::Horizontal => {
//                 match self.alignment.h_align {
//                     AlignH::Left => offset_x,
//                     AlignH::Center => offset_x - (final_width / 2),
//                     AlignH::Right => offset_x - final_width,
//                 }
//             }
//         }
//     }

//     fn start_y(&self, total_height: u32, len: u32) -> u32 {
//         let offset_y = self.offset_y();
//         let stretch = self.spacing * (len - 1);
//         let final_height = total_height + stretch;

//         match self.orientation {
//             Orientation::Vertical => {
//                 match self.alignment.v_align {
//                     AlignV::Top => offset_y,
//                     AlignV::Middle => offset_y - (final_height / 2),
//                     AlignV::Bottom => offset_y - final_height,
//                 }
//             }
//             Orientation::Horizontal => offset_y,
//         }
//     }
// }

// pub(crate) struct LayoutContext {
//     entity: ViewId,
//     next_pos: Vec2u,
//     rules: Rules,
// }

// impl LayoutContext {
//     pub(crate) fn new(entity: ViewId) -> Self {
//         let rules = VIEW_STORAGE.with(|s| {
//             let widget_state = s.get_widget_state(&entity);
//             Rules::new(&widget_state)
//         });
//         Self {
//             entity,
//             next_pos: Vec2u::new(0, 0),
//             rules,
//         }
//     }

//     fn initialize_next_pos(&mut self, children: Option<&Vec<ViewId>>) {
//         let (stretch_factor, child_len) = children.map(|c| {(
//             c.iter()
//                 .map(|child| {
//                     let rect = VIEW_STORAGE.with(|s| {
//                         s.get_widget_state(child).rect
//                     });
//                     rect.read_untracked(|rect| {
//                         match self.rules.orientation {
//                             Orientation::Vertical => rect.size().height(),
//                             Orientation::Horizontal => rect.size().width(),
//                         }
//                     })
//                 })
//                 .sum(),
//                 c.len() as u32)
//         }).unwrap_or_default();

//         match self.rules.orientation {
//             Orientation::Vertical => {
//                 self.next_pos.set_x(self.rules.offset_x());
//                 self.next_pos.set_y(self.rules.start_y(stretch_factor, child_len));
//             }
//             Orientation::Horizontal => {
//                 self.next_pos.set_x(self.rules.start_x(stretch_factor, child_len));
//                 self.next_pos.set_y(self.rules.offset_y());
//             }
//         }
//     }

//     fn assign_position(&mut self, child: &ViewId) {
//         let rect = VIEW_STORAGE.with(|s| {
//             s.get_widget_state(child).rect
//         });
//         let size = rect.read_untracked(|rect| rect.size());

//         match self.rules.orientation {
//             Orientation::Vertical => {
//                 self.next_pos.add_y(size.height() / 2);
//                 match self.rules.alignment.h_align {
//                     AlignH::Left | AlignH::Right => {
//                         self.next_pos.set_x(self.rules.offset_x());
//                         self.next_pos.set_x(self.next_pos.x() + size.width() / 2)
//                     }
//                     AlignH::Center => {},
//                 }
//             },
//             Orientation::Horizontal => {
//                 self.next_pos.add_x(size.width() / 2);
//                 match self.rules.alignment.v_align {
//                     AlignV::Top | AlignV::Bottom => {
//                         self.next_pos.set_y(self.rules.offset_y());
//                         self.next_pos.set_y(self.next_pos.y() + size.height() / 2);
//                     }
//                     AlignV::Middle => {},
//                 }
//             },
//         }

//         rect.update_untracked(|rect| rect.set_pos(self.next_pos));

//         match self.rules.orientation {
//             Orientation::Vertical => self.next_pos.add_y(self.rules.spacing + size.height() / 2),
//             Orientation::Horizontal => self.next_pos.add_x(self.rules.spacing + size.width() / 2),
//         }
//     }

//     pub(crate) fn calculate(&mut self) {
//         let children = VIEW_STORAGE.with(|s| s.tree.borrow().get_all_children(&self.entity));
//         self.initialize_next_pos(children.as_ref());

//         if let Some(children) = children {
//             children.iter().for_each(|child| {
//                 self.assign_position(child);
//             });
//             children.iter().for_each(|child| {
//                 Self::new(*child).calculate();
//             });
//         }
//     }
// }
