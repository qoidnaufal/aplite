use std::collections::HashMap;

use util::{Size, Vector2};
use crate::{element::Attributes, NodeId};


#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlignment {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlignment {
    Top,
    #[default]
    Middle,
    Bottom,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    horizontal: HAlignment,
    vertical: VAlignment,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutCtx {
    next_pos: Vector2<u32>,
    orientation_storage: HashMap<NodeId, Orientation>,
    alignment_storage: HashMap<NodeId, Alignment>,
    spacing_storage: HashMap<NodeId, u32>,
    padding_storage: HashMap<NodeId, u32>,
    orientation: Orientation,
    spacing: u32,
    padding: u32,
}

impl Default for LayoutCtx {
    fn default() -> Self {
        Self {
            next_pos: Vector2::new(0, 0),
            orientation_storage: HashMap::new(),
            alignment_storage: HashMap::new(),
            spacing_storage: HashMap::new(),
            padding_storage: HashMap::new(),
            orientation: Orientation::Vertical,
            spacing: 0,
            padding: 0,
        }
    }
}

impl LayoutCtx {
    pub fn new() -> Self { Self::default() }

    pub fn insert_alignment(&mut self, node_id: NodeId, alignment: Orientation) {
        self.orientation_storage.insert(node_id, alignment);
    }

    pub fn insert_spacing(&mut self, node_id: NodeId, spacing: u32) {
        self.spacing_storage.insert(node_id, spacing);
    }

    pub fn insert_padding(&mut self, node_id: NodeId, padding: u32) {
        self.padding_storage.insert(node_id, padding);
    }

    pub fn get_spacing(&self) -> u32 { self.spacing }

    pub fn get_padding(&self) -> u32 { self.padding }

    pub fn set_to_parent_orientation(&mut self, parent_id: NodeId) {
        self.orientation = self.orientation_storage[&parent_id];
    }

    fn get_parent_orientation(&self, parent_id: NodeId) -> Option<&Orientation> {
        self.orientation_storage.get(&parent_id)
    }

    pub fn align_vertically(&mut self) {
        self.orientation = Orientation::Vertical;
    }

    pub fn align_horizontally(&mut self) {
        self.orientation = Orientation::Horizontal;
    }

    pub fn set_next_pos<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.next_pos);
    }

    pub fn set_spacing(&mut self, node_id: &NodeId) {
        self.spacing = self.spacing_storage[node_id];
    }

    pub fn set_padding(&mut self, node_id: &NodeId) {
        self.padding = self.padding_storage[node_id];
    }

    pub fn assign_position(&mut self, attribs: &mut Attributes) {
        let half = attribs.dims / 2;
        attribs.pos = self.next_pos + half;
        let spacing = self.spacing;
        match self.orientation {
            Orientation::Vertical => {
                self.set_next_pos(|p| p.y = attribs.pos.y + half.height + spacing);
            }
            Orientation::Horizontal => {
                self.set_next_pos(|p| p.x = attribs.pos.x + half.width + spacing);
            }
        }
    }

    pub fn reset_to_parent(
        &mut self,
        parent_id: NodeId,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        self.set_to_parent_orientation(parent_id);
        let padding = self.padding;
        match self.orientation {
            Orientation::Vertical => {
                self.set_next_pos(|pos| {
                    pos.x = current_pos.x - half.width;
                    pos.y = current_pos.y + half.height + padding;
                });
            }
            Orientation::Horizontal => {
                self.set_next_pos(|pos| {
                    pos.y = current_pos.y - half.height;
                    pos.x = current_pos.x + half.width + padding;
                });
            }
        }
    }
}
