use std::collections::HashMap;

use shared::{Size, Vector2};

// mod data;
pub mod layout;
pub(crate) mod properties;
pub(crate) mod cursor;
pub(crate) mod tree;

use crate::renderer::texture::ImageData;
use crate::renderer::util::Render;
use crate::renderer::Renderer;

use properties::{AspectRatio, Properties};
use tree::{Entity, NodeId, Tree};
use cursor::{Cursor, MouseAction, MouseButton};
use layout::{
    LayoutContext,
    Orientation,
};

pub(crate) enum UpdateMode {
    HoverColor(NodeId),
    ClickColor(NodeId),
    RevertColor(NodeId),
    Transform(NodeId),
}

pub struct Context {
    current: Option<NodeId>,
    pub(crate) tree: Tree<NodeId>,
    pub(crate) properties: Vec<Properties>,
    image_fn: HashMap<NodeId, Box<dyn Fn() -> ImageData>>,
    // pub(crate) data: Data,
    style_fn: HashMap<NodeId, Box<dyn Fn(&mut Properties)>>,
    callbacks: HashMap<NodeId, Box<dyn Fn()>>,
    pub(crate) cursor: Cursor,
    pending_update: Vec<UpdateMode>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            current: None,
            tree: Default::default(),
            properties: Vec::with_capacity(1024),
            image_fn: HashMap::new(),
            // data: Data::default(),
            style_fn: HashMap::new(),
            callbacks: HashMap::new(),
            cursor: Cursor::new(),
            pending_update: Vec::with_capacity(10),
        }
    }
}

// debug
#[cfg(feature = "debug_tree")]
impl Context {
    pub(crate) fn debug_tree(&self) {
        self.print_children_from(NodeId::root());
    }

    pub(crate) fn print_children_from(&self, start: NodeId) {
        eprintln!(" > {start:?}: {:?}", self.get_window_properties().name().unwrap_or_default());
        if start == NodeId::root() {
            self.recursive_print(None, 0);
        } else {
            self.recursive_print(Some(start), 0);
        }
    }

    fn recursive_print(&self, start: Option<NodeId>, indent: usize) {
        let acc = 3;
        if let Some(current) = start {
            if let Some(children) = self.tree.get_all_children(&current) {
                children.iter().for_each(|child| {
                    let prop = self.get_node_data(child);
                    let data = prop.size();
                    let name = prop.name().unwrap_or_default();
                    if self.tree.get_parent(child).is_some_and(|p| self.tree.get_parent(p).is_some()) {
                        for i in 0..(indent - acc)/acc {
                            let c = acc - i;
                            eprint!("{:c$}|", "");
                        }
                        let j = acc - 1;
                        eprintln!("{:j$}╰─ {child:?}: {name:?} | {data:?}", "");
                    } else {
                        eprintln!("{:indent$}╰─ {child:?}: {name:?} | {data:?}", "");
                    }
                    if self.tree.get_first_child(child).is_some() {
                        self.recursive_print(Some(*child), indent + acc);
                    }
                });
            }
        } else {
            self.tree.get_all_ancestor()
                .iter()
                .for_each(|node| {
                    let prop = self.get_node_data(*node);
                    let data = prop.size();
                    let name = prop.name().unwrap_or_default();
                    eprintln!(" > {node:?}: {name:?} | {data:?}");
                    if self.tree.get_first_child(node).is_some() {
                        self.recursive_print(Some(**node), indent + acc);
                    }
                });
        }
    }
}

// render
impl Context {
    pub(crate) fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub(crate) fn submit_update(&mut self, renderer: &mut Renderer) {
        self.pending_update.iter().for_each(|mode| {
            match mode {
                UpdateMode::HoverColor(node_id) => {
                    if let Some(color) = self.get_node_data(&node_id).hover_color() {
                        renderer.update_element_color(node_id.index() - 1, color);
                    }
                },
                UpdateMode::ClickColor(node_id) => {
                    if let Some(color) = self.get_node_data(&node_id).click_color() {
                        renderer.update_element_color(node_id.index() - 1, color);
                    }
                }
                UpdateMode::RevertColor(node_id) => {
                    let color = self.get_node_data(&node_id).fill_color();
                    renderer.update_element_color(node_id.index() - 1, color);
                },
                UpdateMode::Transform(node_id) => {
                    let rect = self.get_node_data(&node_id).rect();

                    renderer.update_element_transform(node_id.index() - 1, rect);
                    renderer.update_element_size(node_id.index() - 1, rect.size());
                }
            }
        });
        self.pending_update.clear();
        renderer.write_data();
    }
}

// window
impl Context {
    pub(crate) fn initialize_root(&mut self, size: Size<u32>) {
        self.tree.insert(NodeId::root(), None);
        self.properties.push(Properties::window_properties(size));
    }

    pub(crate) fn get_window_properties(&self) -> &Properties {
        &self.properties[0]
    }

    pub(crate) fn update_window_properties<F: Fn(&mut Properties)>(&mut self, f: F) {
        if let Some(prop) = self.properties.get_mut(0) {
            f(prop);
        }
    }
}

// data
impl Context {
    pub(crate) fn create_entity(&self) -> NodeId {
        self.tree.create_entity()
    }

    pub(crate) fn current_entity(&self) -> Option<NodeId> {
        self.current
    }

    pub(crate) fn set_current_entity(&mut self, entity: Option<NodeId>) {
        self.current = entity;
    }

    pub(crate) fn insert(
        &mut self,
        node_id: NodeId,
        parent: Option<NodeId>,
        properties: Properties,
    ) {
        self.tree.insert(node_id, parent);
        // self.data.insert(&properties);
        self.properties.push(properties);
    }

    pub(crate) fn add_image<F: Fn() -> ImageData + 'static>(&mut self, node_id: NodeId, f: F) {
        self.image_fn.insert(node_id, Box::new(f));
    }

    pub(crate) fn add_style_fn<F: Fn(&mut Properties) + 'static>(&mut self, node_id: NodeId, style_fn: F) {
        self.style_fn.insert(node_id, Box::new(style_fn));
    }

    pub(crate) fn add_callbacks<F: Fn() + 'static>(&mut self, node_id: NodeId, callback: F) {
        self.callbacks.insert(node_id, Box::new(callback));
    }

    pub(crate) fn get_node_data(&self, node_id: &NodeId) -> &Properties {
        &self.properties[node_id.index()]
    }

    pub(crate) fn get_node_data_mut(&mut self, node_id: &NodeId) -> &mut Properties {
        &mut self.properties[node_id.index()]
    }
}

// layout
impl Context {
    pub(crate) fn layout(&mut self) {
        let ancestors = self.tree
            .get_all_ancestor()
            .iter()
            .map(|node_id| **node_id)
            .collect::<Vec<_>>();

        ancestors
            .iter()
            .for_each(|node_id| {
                self.calculate_size_recursive(node_id);
            });

        self.recursive_layout(&NodeId::root());
    }

    pub(crate) fn recursive_layout(&mut self, node_id: &NodeId) {
        let children = LayoutContext::new(node_id, self).calculate();
        if node_id !=&NodeId::root() { self.pending_update.push(UpdateMode::Transform(*node_id)) }
        if let Some(children) = children {
            children.iter().for_each(|child| self.recursive_layout(child));
        }
    }

    fn calculate_size_recursive(&mut self, node_id: &NodeId) -> Size<u32> {
        let prop = *self.get_node_data(node_id);
        let padding = prop.padding();
        let mut size = prop.size();

        if let Some(children) = self.tree.get_all_children(node_id) {
            children.iter().for_each(|child_id| {
                let child_size = self.calculate_size_recursive(child_id);
                match prop.orientation() {
                    Orientation::Vertical => {
                        size.add_height(child_size.height());
                        size.set_width(size.width().max(child_size.width() + padding.horizontal()));
                    }
                    Orientation::Horizontal => {
                        size.set_height(size.height().max(child_size.height() + padding.vertical()));
                        size.add_width(child_size.width());
                    }
                }
            });
            let child_len = children.len() as u32;
            let stretch = prop.spacing() * (child_len - 1);
            match prop.orientation() {
                Orientation::Vertical => {
                    size.add_height(padding.vertical() + stretch);
                },
                Orientation::Horizontal => {
                    size.add_width(padding.horizontal() + stretch);
                },
            }
        }

        if let AspectRatio::Defined(tuple) = prop.image_aspect_ratio() {
            if let Some(parent) = self.tree.get_parent(node_id) {
                match self.get_node_data(parent).orientation() {
                    Orientation::Vertical => size.adjust_height(tuple.into()),
                    Orientation::Horizontal => size.adjust_width(tuple.into()),
                }
            } else {
                size.adjust_width(tuple.into());
            }
        }

        let final_size = size
            .max(prop.min_width(), prop.min_height())
            .min(prop.max_width(), prop.max_height());

        self.get_node_data_mut(node_id).set_size(final_size);
        final_size
    }
}

// cursor
impl Context {
    pub(crate) fn detect_hovered_ancestor(&mut self) {
        if let Some(current) = self.cursor.hover.curr.as_ref() {
            if self.cursor.ancestor.as_ref() == self.tree.get_ancestor(current) { return }
        }
        self.cursor.ancestor = self
            .tree
            .get_all_ancestor()
            .iter()
            .find_map(|ancestor| {
                if self.get_node_data(ancestor).is_hovered(&self.cursor) {
                    Some(**ancestor)
                } else {
                    None
                }
            })
    }

    fn detect_hover_recursive(&self, node_id: &NodeId, acc: &mut Vec<NodeId>) {
        if let Some(children) = self.tree.get_all_children(node_id) {
            children
                .iter()
                .filter(|child| self.get_node_data(child).is_hovered(&self.cursor))
                .for_each(|child| {
                    acc.push(*child);
                    self.detect_hover_recursive(child, acc);
                });
        }
    }

    pub(crate) fn detect_hovered_child(&mut self) {
        #[cfg(feature = "stats")] let start = std::time::Instant::now();

        // FIXME: idk if recursive is the best practice here
        let hovered = self.cursor.ancestor
            .map(|ancestor| {
                let mut sub_children = vec![ancestor];
                self.detect_hover_recursive(&ancestor, &mut sub_children);
                sub_children
            })
            .and_then(|children| {
                children.iter().filter_map(|node| {
                    let prop = self.get_node_data(node);
                    if prop.is_hovered(&self.cursor) {
                        Some(*node)
                    } else {
                        None
                    }
                }).max()
            });

        // let hovered = self.tree.iter().skip(1).filter_map(|node| {
        //     let prop = self.get_node_data(node.id());
        //     if prop.is_hovered(cursor) {
        //         Some(*node.id())
        //     } else { None }
        // }).max();

        #[cfg(feature = "stats")] eprint!("{:?}\r", start.elapsed());

        if let Some(id) = hovered {
            if self.cursor.click.obj.is_none() {
                self.cursor.hover.prev = self.cursor.hover.curr;
                self.cursor.hover.curr = Some(id);
            }
        } else {
            self.cursor.hover.prev = self.cursor.hover.curr.take();
        }
    }
}

// event handling
impl Context {
    pub(crate) fn handle_hover(&mut self) {
        if self.cursor.is_hovering_same_obj() && self.cursor.click.obj.is_none() {
            return;
        }
        if let Some(prev_id) = self.cursor.hover.prev.take() {
            self.pending_update.push(UpdateMode::RevertColor(prev_id));
        }
        if let Some(hover_id) = self.cursor.hover.curr {
            self.pending_update.push(UpdateMode::HoverColor(hover_id));
            let dragable = self.get_node_data(&hover_id).is_dragable();
            if self.cursor.is_dragging(&hover_id) && dragable {
                self.handle_drag(&hover_id);
            }
        }
    }

    fn handle_drag(&mut self, hover_id: &NodeId) {
        let pos = self.cursor.hover.pos - self.cursor.click.offset;
        self.get_node_data_mut(hover_id).set_position(pos.into());
        self.recursive_layout(hover_id);
    }

    pub(crate) fn handle_click(&mut self, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        self.cursor.set_click_state(action.into(), button.into());
        if let Some(click_id) = self.cursor.click.obj {
            if let Some(callback) = self.callbacks.get(&click_id) {
                callback();
            }
            let props = self.get_node_data(&click_id);
            self.cursor.click.offset = self.cursor.click.pos - Vector2::<f32>::from(props.pos());
            self.pending_update.push(UpdateMode::ClickColor(click_id));
        }
        if self.cursor.state.action == MouseAction::Released {
            if let Some(hover_id) = self.cursor.hover.curr {
                self.pending_update.push(UpdateMode::HoverColor(hover_id));
            }
        }
    }
}

impl Render for Context {
    fn render(&mut self, renderer: &mut Renderer) {
        let nodes = self.tree.iter().skip(1).map(|node| *node.id()).collect::<Vec<_>>();
        nodes.iter().for_each(|node_id| {
            if let Some(image_fn) = self.image_fn.get(node_id) {
                let parent_orientation = self
                    .tree
                    .get_parent(node_id)
                    .map(|parent| self.get_node_data(&parent).orientation());
                let info = renderer.add_texture(image_fn);
                let prop = self.get_node_data_mut(node_id);
                prop.set_texture_id(info.id);
                if prop.image_aspect_ratio().is_source() {
                    if let Some(orientation) = parent_orientation {
                        match orientation {
                            Orientation::Vertical => prop.adjust_height(info.aspect_ratio),
                            Orientation::Horizontal => prop.adjust_width(info.aspect_ratio),
                        }
                    } else {
                        prop.adjust_width(info.aspect_ratio);
                    }
                }
            }

            let prop = self.get_node_data(node_id);
            renderer.add_component(prop);
        });
    }
}
