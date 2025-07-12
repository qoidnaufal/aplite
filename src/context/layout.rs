use aplite_reactive::*;
use aplite_types::{Rect, Vec2u};

use crate::widget_state::WidgetState;
use crate::view::{ViewId, VIEW_STORAGE};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub(crate) h_align: HAlign,
    pub(crate) v_align: VAlign,
}

impl Alignment {
    pub const fn new() -> Self {
        Self {
            h_align: HAlign::Center,
            v_align: VAlign::Middle,
        }
    }

    pub fn set_h(&mut self, h_align: HAlign) {
        self.h_align = h_align;
    }

    pub fn set_v(&mut self, v_align: VAlign) {
        self.v_align = v_align;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Padding {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32,
}

impl Padding {
    pub const fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub const fn all(value: u32) -> Self {
        Self {
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub(crate) fn vertical(&self) -> u32 { self.top() + self.bottom() }

    pub(crate) fn horizontal(&self) -> u32 { self.left() + self.right() }

    pub(crate) fn top(&self) -> u32 { self.top }

    pub(crate) fn bottom(&self) -> u32 { self.bottom }

    pub(crate) fn left(&self) -> u32 { self.left }

    pub(crate) fn right(&self) -> u32 { self.right }

    pub fn set_top(&mut self, value: u32) { self.top = value }

    pub fn set_bottom(&mut self, value: u32) { self.bottom = value }

    pub fn set_left(&mut self, value: u32) { self.left = value }

    pub fn set_right(&mut self, value: u32) { self.right = value }

    pub fn set_all(&mut self, value: u32) {
        self.top = value;
        self.bottom = value;
        self.left = value;
        self.right = value;
    }
}

#[derive(Debug)]
pub(crate) struct Rules {
    rect: Rect<u32>,
    orientation: Orientation,
    alignment: Alignment,
    padding: Padding,
    spacing: u32,
}

impl Rules {
    pub(crate) fn new(state: &WidgetState) -> Self {
        Self {
            rect: state.rect.get_untracked(),
            orientation: state.orientation(),
            alignment: state.alignment(),
            padding: state.padding(),
            spacing: state.spacing(),
        }
    }
}

impl Rules {
    fn offset_x(&self) -> u32 {
        let pl = self.padding.left();
        let pr = self.padding.right();

        match self.alignment.h_align {
            HAlign::Left => self.rect.x() - (self.rect.width() / 2) + pl,
            HAlign::Center => {
                if pl >= pr {
                    self.rect.x() + (pl - pr) / 2
                } else {
                    self.rect.x() - (pr - pl) / 2
                }
            }
            HAlign::Right => self.rect.x() + (self.rect.width() / 2) - pr
        }
    }

    fn offset_y(&self) -> u32 {
        let pt = self.padding.top();
        let pb = self.padding.bottom();

        match self.alignment.v_align {
            VAlign::Top => self.rect.y() - (self.rect.height() / 2) + pt,
            VAlign::Middle => {
                if pt >= pb {
                    self.rect.y() + (pt - pb) / 2
                } else {
                    self.rect.y() - (pb - pt) / 2
                }
            }
            VAlign::Bottom => self.rect.y() + (self.rect.height() / 2) - pb,
        }
    }

    fn start_x(&self, total_width: u32, len: u32) -> u32 {
        let offset_x = self.offset_x();
        let stretch = self.spacing * (len - 1);
        let final_width = total_width + stretch;

        match self.orientation {
            Orientation::Vertical => offset_x,
            Orientation::Horizontal => {
                match self.alignment.h_align {
                    HAlign::Left => offset_x,
                    HAlign::Center => offset_x - (final_width / 2),
                    HAlign::Right => offset_x - final_width,
                }
            }
        }
    }

    fn start_y(&self, total_height: u32, len: u32) -> u32 {
        let offset_y = self.offset_y();
        let stretch = self.spacing * (len - 1);
        let final_height = total_height + stretch;

        match self.orientation {
            Orientation::Vertical => {
                match self.alignment.v_align {
                    VAlign::Top => offset_y,
                    VAlign::Middle => offset_y - (final_height / 2),
                    VAlign::Bottom => offset_y - final_height,
                }
            }
            Orientation::Horizontal => offset_y,
        }
    }
}

pub(crate) struct LayoutContext<'a> {
    entity: &'a ViewId,
    next_pos: Vec2u,
    rules: Rules,
}

impl<'a> LayoutContext<'a> {
    pub(crate) fn new(entity: &'a ViewId) -> Self {
        let rules = VIEW_STORAGE.with(|s| {
            let widget_state = s.get_widget_state(entity);
            Rules::new(&widget_state)
        });
        Self {
            entity,
            next_pos: Vec2u::new(0, 0),
            rules,
        }
    }

    fn query_children(&mut self, children: Option<&Vec<ViewId>>) -> (u32, u32) {
        if let Some(children) = children {
            (children.iter().map(|child| {
                let rect = VIEW_STORAGE.with(|s| {
                    s.get_widget_state(child).rect
                });
                rect.read_untracked(|rect| {
                    match self.rules.orientation {
                        Orientation::Vertical => rect.size().height(),
                        Orientation::Horizontal => rect.size().width(),
                    }
                })
            }).sum(), children.len() as u32)
        } else { (0, 0) }
    }

    fn initialize_next_pos(&mut self, children: Option<&Vec<ViewId>>) {
        let (child_size, child_len) = self.query_children(children);

        match self.rules.orientation {
            Orientation::Vertical => {
                self.next_pos.set_x(self.rules.offset_x());
                self.next_pos.set_y(self.rules.start_y(child_size, child_len));
            }
            Orientation::Horizontal => {
                self.next_pos.set_x(self.rules.start_x(child_size, child_len));
                self.next_pos.set_y(self.rules.offset_y());
            }
        }
    }

    // fn get_children(&self) -> Option<Vec<ViewId>> {
    //     VIEW_STORAGE.with(|s| {
    //         let tree = s.tree.borrow();
    //         tree.get_all_children(self.entity)
    //             .map(|children| {
    //                 children.iter()
    //                     .map(|id| **id)
    //                     .collect::<Vec<_>>()
    //             })
    //     })
    // }

    fn assign_position(&mut self, child: &ViewId) {
        let rect = VIEW_STORAGE.with(|s| {
            s.get_widget_state(child).rect
        });
        let size = rect.read_untracked(|rect| rect.size());

        match self.rules.orientation {
            Orientation::Vertical => {
                self.next_pos.add_y(size.height() / 2);
                match self.rules.alignment.h_align {
                    HAlign::Left | HAlign::Right => {
                        self.next_pos.set_x(self.rules.offset_x());
                        self.next_pos.set_x(self.next_pos.x() + size.width() / 2)
                    }
                    HAlign::Center => {},
                }
            },
            Orientation::Horizontal => {
                self.next_pos.add_x(size.width() / 2);
                match self.rules.alignment.v_align {
                    VAlign::Top | VAlign::Bottom => {
                        self.next_pos.set_y(self.rules.offset_y());
                        self.next_pos.set_y(self.next_pos.y() + size.height() / 2);
                    }
                    VAlign::Middle => {},
                }
            },
        }

        rect.update_untracked(|rect| rect.set_pos(self.next_pos));

        match self.rules.orientation {
            Orientation::Vertical => self.next_pos.add_y(self.rules.spacing + size.height() / 2),
            Orientation::Horizontal => self.next_pos.add_x(self.rules.spacing + size.width() / 2),
        }
    }

    pub(crate) fn calculate(&mut self) -> Option<Vec<ViewId>> {
        let children = VIEW_STORAGE.with(|s| {
            s.tree
                .borrow()
                .get_all_children(self.entity)
                .map(|vec| {
                    vec.iter()
                        .map(|id| **id)
                        .collect::<Vec<_>>()
                })
        });
        self.initialize_next_pos(children.as_ref());

        if let Some(children) = children.as_ref() {
            children.iter().for_each(|child| {
                self.assign_position(child);
            });
        }

        children
    }
}
