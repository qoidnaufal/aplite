use std::collections::HashMap;
use util::Vector2;

use crate::context::{Cursor, LayoutCtx, MouseAction};
use crate::renderer::{Gfx, Renderer};
use crate::view::NodeId;
use crate::callback::CALLBACKS;
use crate::Rgb;

#[derive(Debug)]
pub struct WidgetStorage {
    pub nodes: Vec<NodeId>,
    pub children: HashMap<NodeId, Vec<NodeId>>,
    pub parent: HashMap<NodeId, NodeId>,
    pub cached_color: HashMap<NodeId, Rgb<u8>>,
    pub layout: LayoutCtx,
    pending_update: Vec<NodeId>,
}

impl WidgetStorage {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            cached_color: HashMap::new(),
            layout: LayoutCtx::new(),
            pending_update: Vec::new(),
        }
    }

    pub fn insert_children(&mut self, node_id: NodeId, child_id: NodeId) {
        if let Some(child_vec) = self.children.get_mut(&node_id) {
            child_vec.push(child_id);
        } else {
            self.children.insert(node_id, vec![child_id]);
        }
    }

    pub fn insert_parent(&mut self, node_id: NodeId, parent_id: NodeId) {
        self.parent.insert(node_id, parent_id);
    }

    pub fn get_parent(&self, node_id: NodeId) -> Option<&NodeId> {
        self.parent.get(&node_id)
    }

    pub fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub fn submit_update(&mut self, renderer: &mut Renderer) {
        while let Some(ref change_id) = self.pending_update.pop() {
            let index = self.nodes.iter().position(|node_id| node_id == change_id).unwrap();
            renderer.update(index);
        }
    }

    pub fn detect_hover(&self, cursor: &mut Cursor, gfx: &Gfx) {
        // let start = std::time::Instant::now();
        let hovered = self.nodes.iter().enumerate().filter_map(|(idx, node_id)| {
            let shape = &gfx.shapes.data[idx];
            if shape.is_hovered(cursor) {
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
            if let Some(cached_color) = self.cached_color.get(prev_id) {
                let idx = self.nodes.iter().position(|node_id| node_id == prev_id).unwrap();
                gfx.shapes.update(idx, |shape| shape.revert_color(*cached_color));
                self.pending_update.push(*prev_id);
            }
        }
        if let Some(ref hover_id) = cursor.hover.curr {
            let idx = self.nodes.iter().position(|node_id| node_id == hover_id).unwrap();
            gfx.shapes.update(idx, |shape| {
                CALLBACKS.with_borrow_mut(|callbacks| {
                    if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                        on_hover(shape);
                    }
                    if cursor.is_dragging(*hover_id) {
                        if let Some(on_drag) = callbacks.on_drag.get_mut(hover_id) {
                            on_drag(shape);
                            gfx.transforms.update(idx, |transform| {
                                shape.set_position(cursor, transform);
                            });
                        }
                    }
                });
            });
            self.pending_update.push(*hover_id);
        }
    }

    pub fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            let idx = self.nodes.iter().position(|node_id| node_id == click_id).unwrap();
            let shape = gfx.shapes.data.get_mut(idx).unwrap();
            cursor.click.delta = cursor.click.pos - Vector2::<f32>::from(shape.pos);
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_click) = callbacks.on_click.get_mut(click_id) {
                    on_click(shape);
                    self.pending_update.push(*click_id);
                }
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = self.nodes.iter().position(|node_id| node_id == hover_id).unwrap();
                let shape = gfx.shapes.data.get_mut(idx).unwrap();
                CALLBACKS.with_borrow_mut(|callbacks| {
                    if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                        on_hover(shape);
                        self.pending_update.push(*hover_id);
                    }
                });
            }
        }
    }
}

