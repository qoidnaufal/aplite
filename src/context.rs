use std::sync::atomic::{AtomicU64, Ordering};
use util::{Matrix4x4, Size, Vector2};

use crate::layout::Layout;
use crate::cursor::{Cursor, MouseAction};
use crate::renderer::{Buffer, Gfx, Renderer};
use crate::element::Element;
use crate::properties::{Orientation, Properties};
use crate::callback::{Callbacks, CALLBACKS};

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
    parent: Option<usize>,
    children: Option<Vec<usize>>,
    data: T,
}

impl<T: Clone> Node<T> {
    fn new(
        id: NodeId,
        parent: Option<usize>,
        data: T
    ) -> Self {
        Self {
            id,
            parent,
            children: None,
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
        maybe_parent: Option<NodeId>,
        // children: Option<Vec<NodeId>>,
        data: T
    ) {
        let len = self.len();
        let mut parent = None;
        if let Some(parent_id) = maybe_parent {
            let maybe_node = self.iter_mut().enumerate().find_map(|(idx, node)| {
                if node.id == parent_id {
                    parent = Some(idx);
                    Some(node)
                } else { None }
            });
            if let Some(parent_node) = maybe_node {
                if let Some(children) = parent_node.children.as_mut() {
                    children.push(len);
                } else {
                    parent_node.children = Some(vec![len]);
                }
            }
        }
        // let parent = self
        //     .iter()
        //     .find(|node| node.children.as_ref().is_some_and(|children| children.contains(&id)))
        //     .map(|node| {
        //         let children = node.children.as_ref().unwrap();
        //         let idx = children.iter().position(|child_id| child_id == &id).unwrap();
        //         if idx > 0 {
        //             prev = children.get(idx - 1).cloned();
        //         }
        //         if idx + 1 < children.len() {
        //             next = children.get(idx + 1).cloned();
        //         }
        //         node.id
        //     });
        let node = Node::new(id, parent, data);
        self.storage.push(node);
    }

    pub(crate) fn get_node_data(&self, id: &NodeId) -> Option<&T> {
        self
            .iter()
            .find_map(|node| {
                if node.id == *id {
                    Some(&node.data)
                } else { None }
            })
    }

    pub(crate) fn get_node_data_mut(&mut self, id: &NodeId) -> Option<&mut T> {
        self
            .iter_mut()
            .find_map(|node| {
                if node.id == *id {
                    Some(&mut node.data)
                } else { None }
            })
    }
}

// impl<T> std::ops::Index<NodeId> for ArenaStorage<Node<T>> {
//     type Output = Node<T>;
//     fn index(&self, index: NodeId) -> &Self::Output {
//         self.iter().find(|node| node.id == index).unwrap()
//     }
// }

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
pub struct Context {
    nodes: ArenaStorage<Node<Properties>>,
    layout: Layout,
    pending_update: Vec<NodeId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            layout: Layout::new(),
            pending_update: Vec::new(),
        }
    }
}

impl Context {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self,
        node_id: NodeId,
        maybe_parent: Option<NodeId>,
        data: &Properties
    ) {
        self.nodes.push(node_id, maybe_parent, *data);
    }

    pub(crate) fn get_parent(&self, node_id: &NodeId) -> Option<&usize> {
        self.nodes.iter().find_map(|node| {
            if node.id == *node_id {
                node.parent.as_ref()
            } else { None }
        })
    }

    // pub(crate) fn next_sibling(&self, node_id: &NodeId) -> Option<&NodeId> {
    //     self.nodes.iter().find_map(|node| {
    //         if node.id == *node_id {
    //             node.next.as_ref()
    //         } else { None }
    //     })
    // }

    // pub(crate) fn prev_sibling(&self, node_id: &NodeId) -> Option<&NodeId> {
    //     self.nodes.iter().find_map(|node| {
    //         if node.id == *node_id {
    //             node.prev.as_ref()
    //         } else { None }
    //     })
    // }

    pub(crate) fn contains(&self, node_id: &NodeId) -> bool {
        self.nodes.iter().find_map(|node| {
            if node.id == *node_id {
                Some(true)
            } else { None }
        }).unwrap_or(false)
    }

    pub(crate) fn is_root(&self, node_id: &NodeId) -> bool {
        self.nodes.iter().find_map(|node| {
            if node.id == *node_id && node.parent.is_none() {
                Some(true)
            } else { None }
        }).unwrap_or(false)
    }

    pub(crate) fn get_node_data(&self, node_id: &NodeId) -> Option<&Properties> {
        self.nodes.get_node_data(node_id)
    }

    pub(crate) fn get_node_data_mut(&mut self, node_id: &NodeId) -> Option<&mut Properties> {
        self.nodes.get_node_data_mut(node_id)
    }

    pub(crate) fn set_orientation(&mut self, node_id: &NodeId) {
        if let Some(style) = self.get_node_data(node_id) {
            self.layout.set_orientation(style.orientation());
        }
    }

    pub(crate) fn set_alignment(&mut self, node_id: &NodeId) {
        if let Some(style) = self.get_node_data(node_id) {
            self.layout.set_alignment(style.alignment());
        }
    }

    pub(crate) fn set_spacing(&mut self, node_id: &NodeId) {
        if let Some(style) = self.get_node_data(node_id) {
            self.layout.set_spacing(style.spacing());
        }
    }

    pub(crate) fn set_padding(&mut self, node_id: &NodeId) {
        if let Some(style) = self.get_node_data(node_id) {
            self.layout.set_padding(style.padding());
        }
    }

    pub(crate) fn set_next_pos(&mut self, f: impl FnOnce(&mut Vector2<u32>)) {
        self.layout.set_next_pos(f);
    }

    pub(crate) fn assign_position(&mut self, node_id: &NodeId) {
        let next_pos = self.layout.next_pos();
        let mut pos: Vector2<u32> = Vector2::default();
        let mut size: Size<u32> = Size::default();
        if let Some(style) = self.get_node_data_mut(node_id) {
            let half = style.size() / 2;
            style.set_position(next_pos + half);
            pos = style.pos();
            size = style.size();
        }

        self.layout.assign_position(pos, size);
    }
    pub(crate) fn reset_to_parent(
        &mut self,
        idx: usize,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        let node_id = self.nodes[idx].id;
        self.set_orientation(&node_id);
        self.set_alignment(&node_id);
        self.set_spacing(&node_id);
        self.set_padding(&node_id);
        self.layout.reset_to_parent(current_pos, half);
    }

    // .....................................................................

    pub(crate) fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub(crate) fn submit_update(&mut self, renderer: &mut Renderer) {
        self.pending_update.clear();
        renderer.update();
    }

    pub(crate) fn detect_hover(&self, cursor: &mut Cursor) {
        // let start = std::time::Instant::now();
        let hovered = self.nodes.iter().filter_map(|node| {
            if node.data.is_hovered(cursor) {
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
            let idx = self.nodes.iter().position(|node| node.id == prev_id).unwrap();
            let properties = &self.nodes.storage[idx].data;
            gfx.elements.update(idx, |element| element.set_color(properties.fill_color()));
            self.pending_update.push(prev_id);
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
            if let Some(properties) = self.nodes.get_node_data_mut(hover_id) {
                transforms.update(element.transform_id as usize, |transform| {
                    let delta = cursor.hover.pos - cursor.click.offset;
                    properties.set_transform(delta, transform);
                });
            }
            let idx = self.nodes.iter().position(|node| node.id == *hover_id).unwrap();
            self.handle_child_relayout(idx, transforms);
        }
    }

    fn handle_child_relayout(
        &mut self,
        node_idx: usize,
        transforms: &mut Buffer<Matrix4x4>,
    ) {
        let node_id = self.nodes[node_idx].id;
        let children = self.nodes[node_idx].children.clone();

        if let Some(children) = children {
            let properties = self.nodes[node_idx].data.clone();
            let pos = properties.pos();
            let size = properties.size();
            self.set_orientation(&node_id);
            self.set_spacing(&node_id);
            self.set_padding(&node_id);
            let padding = {
                let padding = properties.padding();
                match self.layout.orientation() {
                    Orientation::Vertical => padding.top(),
                    Orientation::Horizontal => padding.left(),
                }
            };
            self.layout.set_next_pos(|next_pos| {
                next_pos.x = pos.x - size.width / 2 + padding;
                next_pos.y = pos.y - size.height / 2 + padding;
            });

            children.iter().for_each(|child_idx| {
                let child_id = self.nodes[*child_idx].id;
                transforms.update(*child_idx, |child_transform| {
                    self.assign_position(&child_id);
                    let child_props = &self.nodes[*child_idx].data;
                    let x = child_props.pos().x as f32 / (child_props.size().width as f32 / child_transform[0].x) * 2.0 - 1.0;
                    let y = 1.0 - child_props.pos().y as f32 / (child_props.size().height as f32 / child_transform[1].y) * 2.0;
                    child_transform.translate(x, y);
                });

                self.handle_child_relayout(*child_idx, transforms);
            });

            if let Some(parent_id) = self.nodes[node_idx].parent {
                self.reset_to_parent(parent_id, pos, size / 2);
            }
        }
    }

    pub(crate) fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            let idx = self.nodes.iter().position(|node| node.id == *click_id).unwrap();
            let pos = self.nodes.storage[idx].data.pos();
            cursor.click.offset = cursor.click.pos - Vector2::<f32>::from(pos);
            gfx.elements.update(idx, |element| {
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_click(click_id, element);
                    self.pending_update.push(*click_id);
                });
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = self.nodes.iter().position(|node| node.id == *hover_id).unwrap();
                gfx.elements.update(idx, |element| {
                    CALLBACKS.with_borrow_mut(|callbacks| {
                        callbacks.handle_hover(hover_id, element);
                        self.pending_update.push(*hover_id);
                    });
                });
            }
        }
    }
}

