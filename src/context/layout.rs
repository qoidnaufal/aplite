use shared::{Size, Vector2};

use crate::context::Context;
use crate::properties::Properties;
use crate::tree::NodeId;

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
    pub(crate) const fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self { top, bottom, left, right }
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
    position: Vector2<u32>,
    size: Size<u32>,
    orientation: Orientation,
    alignment: Alignment,
    padding: Padding,
    spacing: u32,
}

impl Rules {
    pub(crate) fn new(prop: &Properties) -> Self {
        Self {
            position: prop.pos(),
            size: prop.size(),
            orientation: prop.orientation(),
            alignment: prop.alignment(),
            padding: prop.padding(),
            spacing: prop.spacing(),
        }
    }
}

impl Rules {
    // fn inner_space(&self) -> Size<u32> {
    //     (self.size.width() - self.padding.horizontal(),
    //     self.size.height() - self.padding.vertical()).into()
    // }

    fn offset_x(&self) -> u32 {
        let pl = self.padding.left();
        let pr = self.padding.right();

        match self.alignment.h_align {
            HAlign::Left => self.position.x() - (self.size.width() / 2) + pl,
            HAlign::Center => {
                if pl >= pr {
                    self.position.x() + (pl - pr) / 2
                } else {
                    self.position.x() - (pr - pl) / 2
                }
            }
            HAlign::Right => self.position.x() + (self.size.width() / 2) - pr
        }
    }

    fn offset_y(&self) -> u32 {
        let pt = self.padding.top();
        let pb = self.padding.bottom();

        match self.alignment.v_align {
            VAlign::Top => self.position.y() - (self.size.height() / 2) + pt,
            VAlign::Middle => {
                if pt >= pb {
                    self.position.y() + (pt - pb) / 2
                } else {
                    self.position.y() - (pb - pt) / 2
                }
            }
            VAlign::Bottom => self.position.y() + (self.size.height() / 2) - pb,
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
    entity: &'a NodeId,
    cx: &'a mut Context,
    next_pos: Vector2<u32>,
    // avalilable_space: Size<u32>,
    rules: Rules,
}

impl<'a> LayoutContext<'a> {
    pub(crate) fn new(entity: &'a NodeId, cx: &'a mut Context) -> Self {
        let rules = Rules::new(cx.get_node_data(&entity));
        Self {
            entity,
            cx,
            // avalilable_space: rules.inner_space(),
            next_pos: Vector2::new(0, 0),
            rules,
        }
    }

    fn query_children(&mut self, children: Option<&Vec<NodeId>>) -> (u32, u32) {
        if let Some(children) = children {
            (children.iter().map(|child| {
                let size = self.cx.get_node_data(child).size();
                match self.rules.orientation {
                    Orientation::Vertical => size.height(),
                    Orientation::Horizontal => size.width(),
                }
            }).sum(), children.len() as u32)
        } else { (0, 0) }
    }

    fn initialize_next_pos(&mut self, children: Option<&Vec<NodeId>>) {
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

    fn get_children(&self) -> Option<Vec<NodeId>> {
        if self.entity == &NodeId::root() {
            self.cx.tree.get_all_ancestor()
                .iter()
                .map(|a| Some(**a))
                .collect::<Option<Vec<_>>>()
        } else {
            self.cx.tree.get_all_children(self.entity)
        }
    }

    fn assign_position(&mut self, child: &NodeId) {
        let prop = self.cx.get_node_data_mut(child);
        let size = prop.size();

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

        prop.set_position(self.next_pos);

        match self.rules.orientation {
            Orientation::Vertical => self.next_pos.add_y(self.rules.spacing + size.height() / 2),
            Orientation::Horizontal => self.next_pos.add_x(self.rules.spacing + size.width() / 2),
        }
    }

    pub(crate) fn calculate(&mut self) -> Option<Vec<NodeId>> {
        let children = self.get_children();
        self.initialize_next_pos(children.as_ref());

        if let Some(children) = children.as_ref() {
            children.iter().for_each(|child| {
                self.assign_position(child);
            });
        }

        children
    }
}
