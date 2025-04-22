use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use util::{Matrix4x4, Vector2};

use crate::layout::Layout;
use crate::cursor::{Cursor, MouseAction};
use crate::renderer::{Buffer, Gfx, Renderer};
use crate::element::Element;
use crate::style::{Orientation, Style};
use crate::callback::{Callbacks, CALLBACKS};
use crate::color::Rgba;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new() -> Self {
        static NODE_ID: AtomicU64 = AtomicU64::new(0);
        Self(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    id: NodeId,
    next: Option<NodeId>,
    prev: Option<NodeId>,
    parent: Option<NodeId>,
    children: Option<Vec<NodeId>>,
    data: T,
}

impl<T: Clone> Node<T> {
    fn new(
        id: NodeId,
        next: Option<NodeId>,
        prev: Option<NodeId>,
        parent: Option<NodeId>,
        children: Option<Vec<NodeId>>,
        data: T
    ) -> Self {
        Self {
            id,
            next,
            prev,
            parent,
            children,
            data,
        }
    }
}

#[derive(Debug)]
pub struct ArenaStorage<N> {
    storage: Vec<N>,
}

impl<N> Default for ArenaStorage<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> ArenaStorage<N> {
    fn new() -> Self {
        Self {
            storage: Vec::with_capacity(1024),
        }
    }
}

impl<T: Clone> ArenaStorage<Node<T>> {
    pub(crate) fn push(
        &mut self,
        id: NodeId,
        children: Option<Vec<NodeId>>,
        data: T
    ) {
        // self.iter_mut().find(|node| node.id == id).map(|node|)
        let mut next = None;
        let mut prev = None;
        let parent = self
            .iter()
            .find(|node| node.children.as_ref().is_some_and(|children| children.contains(&id)))
            .map(|node| {
                if let Some(children) = node.children.as_ref() {
                    let idx = children.iter().position(|child_node| child_node == &id).unwrap();
                    if idx > 0 {
                        prev = children.get(idx - 1).cloned();
                    }
                    if idx + 1 < children.len() {
                        next = children.get(idx + 1).cloned();
                    }
                }
                node.id
            });
        let node = Node::new(id, next, prev, parent, children, data);
        self.storage.push(node);
    }
}

impl<N> std::ops::Deref for ArenaStorage<N> {
    type Target = [N];
    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<N> std::ops::DerefMut for ArenaStorage<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

impl<N> IntoIterator for ArenaStorage<N> {
    type Item = N;
    type IntoIter = std::vec::IntoIter<N>;
    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}

impl<'a, N> IntoIterator for &'a ArenaStorage<N> {
    type Item = &'a N;
    type IntoIter = std::slice::Iter<'a, N>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, N> IntoIterator for &'a mut ArenaStorage<N> {
    type Item = &'a mut N;
    type IntoIter = std::slice::IterMut<'a, N>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(Debug)]
pub struct WidgetTree {
    pub nodes: ArenaStorage<Node<Style>>,
    pub cached_color: HashMap<NodeId, Rgba<u8>>,
    pub layout: Layout,
    pending_update: Vec<NodeId>,
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            cached_color: HashMap::new(),
            layout: Layout::new(),
            pending_update: Vec::new(),
        }
    }
}

impl WidgetTree {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_parent(&self, node_id: &NodeId) -> Option<&NodeId> {
        self.nodes.iter().find_map(|node| {
            if node.id == *node_id {
                node.parent.as_ref()
            } else { None }
        })
    }

    pub(crate) fn next_sibling(&self, node_id: &NodeId) -> Option<&NodeId> {
        self.nodes.iter().find_map(|node| {
            if node.id == *node_id {
                node.next.as_ref()
            } else { None }
        })
    }

    pub(crate) fn prev_sibling(&self, node_id: &NodeId) -> Option<&NodeId> {
        self.nodes.iter().find_map(|node| {
            if node.id == *node_id {
                node.prev.as_ref()
            } else { None }
        })
    }

    pub(crate) fn is_root(&self, node_id: &NodeId) -> bool {
        self
            .nodes
            .iter()
            .find_map(|node| {
                if node.id == *node_id && node.parent.is_none() {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false)
        // !self.parent.contains_key(node_id)
    }

    pub(crate) fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub(crate) fn submit_update(&mut self, renderer: &mut Renderer) {
        self.pending_update.clear();
        renderer.update();
    }

    pub(crate) fn detect_hover(&self, cursor: &mut Cursor, gfx: &Gfx) {
        // let start = std::time::Instant::now();
        let hovered = self.nodes.iter().enumerate().filter_map(|(idx, node)| {
            let element = &gfx.elements.data[idx];
            let attr = self.layout.get_attributes(&node.id);
            if element.is_hovered(cursor, &attr) {
                Some(node.id)
            } else { None }
        }).min();
        // eprintln!("{:?}", start.elapsed());
        if let Some(id) = hovered {
            if cursor.click.obj.is_none() {
                cursor.hover.prev = cursor.hover.curr;
                cursor.hover.curr = Some(id);
            }
        } else {
            cursor.hover.prev = cursor.hover.curr.take();
        }
    }

    pub(crate) fn handle_hover(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(prev_id) = cursor.hover.prev.take() {
            if let Some(cached) = self.cached_color.get(&prev_id) {
                let idx = self.nodes.iter().position(|node| node.id == prev_id).unwrap();
                gfx.elements.update(idx, |element| element.revert_color(*cached));
                self.pending_update.push(prev_id);
            }
        }
        if let Some(hover_id) = cursor.hover.curr {
            let idx = self.nodes.iter().position(|node| node.id == hover_id).unwrap();
            gfx.elements.update(idx, |element| {
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_hover(&hover_id, element);
                    if cursor.is_dragging(hover_id) {
                        self.handle_drag(
                            &hover_id,
                            cursor,
                            callbacks,
                            element,
                            &mut gfx.transforms,
                        );
                    }
                });
            });
            self.pending_update.push(hover_id);
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
            if let Some(attribs) = self.layout.get_attributes_mut(hover_id) {
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
        let children = self.nodes.iter().find_map(|node| {
            if node.id == *node_id {
                node.children.as_ref()
            } else { None }
        }).cloned();

        if let Some(children) = children {
            let attr = self.layout.get_attributes(node_id);
            self.layout.set_orientation(node_id);
            self.layout.set_spacing(node_id);
            self.layout.set_padding(node_id);
            let padding = {
                let padding = self.layout.get_padding(node_id);
                match self.layout.orientation() {
                    Orientation::Vertical => padding.top(),
                    Orientation::Horizontal => padding.left(),
                }
            };
            self.layout.set_next_pos(|next_pos| {
                next_pos.x = attr.pos.x - attr.size.width / 2 + padding;
                next_pos.y = attr.pos.y - attr.size.height / 2 + padding;
            });

            children.iter().for_each(|child_id| {
                let idx = self.nodes.iter().position(|node| node.id == *child_id).unwrap();
                transforms.update(idx, |child_transform| {
                    let c_attr = self.layout.assign_position(child_id);
                    let x = c_attr.pos.x as f32 / (c_attr.size.width as f32 / child_transform[0].x) * 2.0 - 1.0;
                    let y = 1.0 - c_attr.pos.y as f32 / (c_attr.size.height as f32 / child_transform[1].y) * 2.0;
                    child_transform.translate(x, y);
                });

                self.handle_child_relayout(child_id, transforms);
            });

            if let Some(parent_id) = self.get_parent(node_id) {
                self.layout.reset_to_parent(*parent_id, attr.pos, attr.size / 2);
            }
        }
    }

    pub(crate) fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            let idx = self.nodes.iter().position(|node| node.id == *click_id).unwrap();
            let element = gfx.elements.data.get_mut(idx).unwrap();
            let pos = self.layout.get_attributes(click_id).pos;
            cursor.click.offset = cursor.click.pos - Vector2::<f32>::from(pos);
            CALLBACKS.with_borrow_mut(|callbacks| {
                callbacks.handle_click(click_id, element);
                self.pending_update.push(*click_id);
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = self.nodes.iter().position(|node| node.id == *hover_id).unwrap();
                let element = &mut gfx.elements.data[idx];
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_hover(hover_id, element);
                    self.pending_update.push(*hover_id);
                });
            }
        }
    }
}

