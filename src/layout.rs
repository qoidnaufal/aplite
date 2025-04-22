use std::collections::HashMap;

use util::{Matrix4x4, Size, Vector2};
use crate::style::{Alignment, Orientation, Padding};
use crate::tree::NodeId;

#[derive(Debug, Clone, Copy, Default)]
pub struct Attributes {
    pub pos: Vector2<u32>,
    pub size: Size<u32>,
}

impl Attributes {
    pub fn new(size: impl Into<Size<u32>>) -> Self {
        Self {
            pos: Vector2::default(),
            size: size.into(),
        }
    }

    pub fn new_with_pos(
        size: impl Into<Size<u32>>,
        pos: impl Into<Vector2<u32>>
    ) -> Self {
        Self {
            pos: pos.into(),
            size: size.into(),
        }
    }

    pub fn adjust_ratio(&mut self, aspect_ratio: f32) {
        self.size.width = (self.size.height as f32 * aspect_ratio) as u32;
    }

    pub fn get_transform(&self, window_size: Size<u32>) -> Matrix4x4 {
        let mut matrix = Matrix4x4::IDENTITY;
        let ws: Size<f32> = window_size.into();
        let x = self.pos.x as f32 / ws.width * 2.0 - 1.0;
        let y = 1.0 - self.pos.y as f32 / ws.height * 2.0;
        let d: Size<f32> = self.size.into();
        let scale = d / ws;
        matrix.transform(x, y, scale.width, scale.height);
        matrix
    }

    pub fn set_position(
        &mut self,
        new_pos: Vector2<f32>,
        transform: &mut Matrix4x4,
    ) {
        self.pos = new_pos.into();
        let x = self.pos.x as f32 / (self.size.width as f32 / transform[0].x) * 2.0 - 1.0;
        let y = 1.0 - self.pos.y as f32 / (self.size.height as f32 / transform[1].y) * 2.0;
        transform.translate(x, y);
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    attributes: HashMap<NodeId, Attributes>,
    orientation_storage: HashMap<NodeId, Orientation>,
    alignment_storage: HashMap<NodeId, Alignment>,
    spacing_storage: HashMap<NodeId, u32>,
    padding_storage: HashMap<NodeId, Padding>,
    next_pos: Vector2<u32>,
    orientation: Orientation,
    alignment: Alignment,
    spacing: u32,
    padding: Padding,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            attributes: HashMap::new(),
            orientation_storage: HashMap::new(),
            alignment_storage: HashMap::new(),
            spacing_storage: HashMap::new(),
            padding_storage: HashMap::new(),
            next_pos: Vector2::default(),
            orientation: Orientation::default(),
            alignment: Alignment::default(),
            padding: Padding::default(),
            spacing: 0,
        }
    }
}

impl Layout {
    pub(crate) fn new() -> Self { Self::default() }

    pub(crate) fn insert_attributes(&mut self, node_id: NodeId, size: Size<u32>) {
        let attributes = Attributes::new(size);
        self.attributes.insert(node_id, attributes);
    }

    pub(crate) fn insert_orientation(&mut self, node_id: NodeId, orientation: Orientation) {
        self.orientation_storage.insert(node_id, orientation);
    }

    pub(crate) fn insert_alignment(&mut self, node_id: NodeId, alignment: Alignment) {
        self.alignment_storage.insert(node_id, alignment);
    }

    pub(crate) fn insert_spacing(&mut self, node_id: NodeId, spacing: u32) {
        self.spacing_storage.insert(node_id, spacing);
    }

    pub(crate) fn insert_padding(&mut self, node_id: NodeId, padding: Padding) {
        self.padding_storage.insert(node_id, padding);
    }
}

impl Layout {
    pub(crate) fn alignment(&self) -> Alignment {
        self.alignment
    }

    pub(crate) fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub(crate) fn get_attributes(&self, node_id: &NodeId) -> Attributes {
        self.attributes[node_id]
    }

    pub(crate) fn get_attributes_mut(&mut self, node_id: &NodeId) -> Option<&mut Attributes> {
        self.attributes.get_mut(node_id)
    }

    pub(crate) fn get_padding(&self, node_id: &NodeId) -> Padding {
        self.padding_storage[node_id]
    }

    pub(crate) fn get_spacing(&self, node_id: &NodeId) -> u32 {
        self.spacing_storage[node_id]
    }
}

impl Layout {
    pub(crate) fn set_alignment(&mut self, node_id: &NodeId) {
        self.alignment = self.alignment_storage[node_id];
    }

    pub(crate) fn set_orientation(&mut self, node_id: &NodeId) {
        self.orientation = self.orientation_storage[node_id];
    }

    pub(crate) fn set_next_pos<F: FnMut(&mut Vector2<u32>)>(&mut self, mut f: F) {
        f(&mut self.next_pos);
    }

    pub(crate) fn set_spacing(&mut self, node_id: &NodeId) {
        self.spacing = self.spacing_storage[node_id];
    }

    pub(crate) fn set_padding(&mut self, node_id: &NodeId) {
        self.padding = self.padding_storage[node_id];
    }

    pub(crate) fn assign_position(&mut self, node_id: &NodeId) -> Attributes {
        let spacing = self.spacing;
        if let Some(attribs) = self.attributes.get_mut(node_id) {
            let half = attribs.size / 2;
            attribs.pos = self.next_pos + half;
            let pos = attribs.pos;
            match self.orientation {
                Orientation::Vertical => {
                    self.set_next_pos(|p| p.y = pos.y + half.height + spacing);
                }
                Orientation::Horizontal => {
                    self.set_next_pos(|p| p.x = pos.x + half.width + spacing);
                }
            };
        }

        self.get_attributes(node_id)
    }

    pub(crate) fn reset_to_parent(
        &mut self,
        parent_id: NodeId,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        self.set_orientation(&parent_id);
        self.set_spacing(&parent_id);
        self.set_padding(&parent_id);
        let spacing = self.spacing;

        // parent orientation
        match self.orientation {
            Orientation::Vertical => {
                self.set_next_pos(|pos| {
                    pos.x = current_pos.x - half.width;
                    pos.y = current_pos.y + half.height + spacing;
                });
            }
            Orientation::Horizontal => {
                self.set_next_pos(|pos| {
                    pos.y = current_pos.y - half.height;
                    pos.x = current_pos.x + half.width + spacing;
                });
            }
        }
    }
}
