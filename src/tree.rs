use std::collections::HashMap;
use util::{Matrix4x4, Vector2};

use crate::context::{Cursor, LayoutCtx, MouseAction};
use crate::renderer::{Buffer, Gfx, Renderer};
use crate::element::{Attributes, Element};
use crate::view::NodeId;
use crate::callback::{Callbacks, CALLBACKS};
use crate::Rgba;

#[derive(Debug)]
pub struct WidgetTree {
    pub nodes: Vec<NodeId>,
    pub children: HashMap<NodeId, Vec<NodeId>>,
    pub parent: HashMap<NodeId, NodeId>,
    pub attribs: HashMap<NodeId, Attributes>,
    pub cached_color: HashMap<NodeId, Rgba<u8>>,
    pub layout: LayoutCtx,
    pending_update: Vec<NodeId>,
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            attribs: HashMap::new(),
            cached_color: HashMap::new(),
            layout: LayoutCtx::new(),
            pending_update: Vec::new(),
        }
    }
}

impl WidgetTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_children(&mut self, node_id: NodeId, child_id: NodeId) {
        let child_vec = self
            .children
            .entry(node_id)
            .or_insert(vec![child_id]);
        if !child_vec.contains(&child_id) {
            child_vec.push(child_id);
        }
    }

    pub fn insert_parent(&mut self, node_id: NodeId, parent_id: NodeId) {
        self.parent.insert(node_id, parent_id);
    }

    pub fn get_parent(&self, node_id: &NodeId) -> Option<&NodeId> {
        self.parent.get(node_id)
    }

    // fn get_children(&self, node_id: NodeId) -> Option<&Vec<NodeId>> {
    //     self.children.get(&node_id)
    // }

    pub fn is_root(&self, node_id: &NodeId) -> bool {
        self.parent.get(&node_id).is_none()
    }

    pub fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub fn submit_update(&mut self, renderer: &mut Renderer) {
        self.pending_update.clear();
        renderer.update();
    }

    pub fn detect_hover(&self, cursor: &mut Cursor, gfx: &Gfx) {
        // let start = std::time::Instant::now();
        let hovered = self.nodes.iter().enumerate().filter_map(|(idx, node_id)| {
            let element = &gfx.elements.data[idx];
            let attr = &self.attribs[node_id];
            if element.is_hovered(cursor, attr) {
                Some(node_id)
            } else { None }
        }).min();
        // eprintln!("{:?}", start.elapsed());
        if let Some(id) = hovered {
            if cursor.click.obj.is_none() {
                cursor.hover.prev = cursor.hover.curr;
                cursor.hover.curr = Some(*id);
            }
        } else {
            cursor.hover.prev = cursor.hover.curr.take();
        }
    }

    pub fn handle_hover(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(ref prev_id) = cursor.hover.prev.take() {
            if let Some(cached) = self.cached_color.get(prev_id) {
                let idx = self.nodes.iter().position(|node_id| node_id == prev_id).unwrap();
                gfx.elements.update(idx, |element| element.revert_color(*cached));
                self.pending_update.push(*prev_id);
            }
        }
        if let Some(ref hover_id) = cursor.hover.curr {
            let idx = self.nodes.iter().position(|node_id| node_id == hover_id).unwrap();
            gfx.elements.update(idx, |element| {
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_hover(hover_id, element);
                    if cursor.is_dragging(*hover_id) {
                        self.handle_drag(
                            hover_id,
                            cursor,
                            callbacks,
                            element,
                            &mut gfx.transforms,
                        );
                    }
                });
            });
            self.pending_update.push(*hover_id);
        }
    }

    fn handle_drag(
        &mut self,
        hover_id: &NodeId,
        cursor: &Cursor,
        callbacks: &mut Callbacks,
        element: &mut Element,
        transforms: &mut Buffer<Matrix4x4>,
    ) {
        if let Some(on_drag) = callbacks.on_drag.get_mut(hover_id) {
            on_drag(element);
            if let Some(attribs) = self.attribs.get_mut(hover_id) {
                transforms.update(element.transform_id as usize, |transform| {
                    let delta = cursor.hover.pos - cursor.click.offset;
                    attribs.set_position(delta, transform);
                });
            }
            self.handle_child_relayout(hover_id, transforms);
        }
    }

    fn handle_child_relayout(
        &mut self,
        node_id: &NodeId,
        transforms: &mut Buffer<Matrix4x4>,
    ) {
        if let Some(children) = self.children.get(node_id).cloned() {
            let attr = self.attribs[node_id];
            let padding = self.layout.get_padding();
            self.layout.set_to_parent_alignment(*node_id);
            self.layout.set_spacing(node_id);
            self.layout.set_padding(node_id);
            self.layout.set_next_pos(|next_pos| {
                next_pos.x = attr.pos.x - attr.dims.width / 2 + padding;
                next_pos.y = attr.pos.y - attr.dims.height / 2 + padding;
            });

            children.iter().for_each(|child_id| {
                let idx = self.nodes.iter().position(|node_id| node_id == child_id).unwrap();
                transforms.update(idx, |child_transform| {
                    if let Some(child_attribs) = self.attribs.get_mut(child_id) {
                        self.layout.assign_position(child_attribs);
                        child_attribs.set_position(child_attribs.pos.into(), child_transform);
                    }
                });

                self.handle_child_relayout(child_id, transforms);
            });

            if let Some(parent_id) = self.get_parent(node_id) {
                self.layout.reset_to_parent(*parent_id, attr.pos, attr.dims / 2);
            }
        }
    }

    pub fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            let idx = self.nodes.iter().position(|node_id| node_id == click_id).unwrap();
            let element = gfx.elements.data.get_mut(idx).unwrap();
            let pos = self.attribs[click_id].pos;
            cursor.click.offset = cursor.click.pos - Vector2::<f32>::from(pos);
            CALLBACKS.with_borrow_mut(|callbacks| {
                callbacks.handle_click(click_id, element);
                self.pending_update.push(*click_id);
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = self.nodes.iter().position(|node_id| node_id == hover_id).unwrap();
                let element = &mut gfx.elements.data[idx];
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_hover(hover_id, element);
                    self.pending_update.push(*hover_id);
                });
            }
        }
    }
}

