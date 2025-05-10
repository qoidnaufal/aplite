use std::collections::HashMap;

// use data::Data;
use shared::{Size, Vector2};

use crate::color::Pixel;
use crate::cursor::{Cursor, MouseAction};
use crate::renderer::{Gfx, IntoRenderSource, RenderComponentSource, Renderer};
use crate::properties::Properties;
use crate::tree::{Entity, NodeId, Tree};

// mod data;
pub mod layout;

use layout::{
    LayoutContext,
    Orientation,
};

pub struct Context {
    current: Option<NodeId>,
    pub(crate) tree: Tree<NodeId>,
    pub(crate) debug_name: Vec<Option<&'static str>>,
    properties: Vec<Properties>,
    pixels: Vec<Pixel<u8>>,
    // pub(crate) data: Data,
    style_fn: HashMap<NodeId, Box<dyn Fn(&mut Properties)>>,
    callbacks: HashMap<NodeId, Box<dyn Fn()>>,
    pending_update: Vec<NodeId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            current: None,
            tree: Default::default(),
            debug_name: Vec::with_capacity(1024),
            properties: Vec::with_capacity(1024),
            pixels: Vec::new(),
            // data: Data::default(),
            style_fn: HashMap::new(),
            callbacks: HashMap::new(),
            pending_update: Vec::with_capacity(10),
        }
    }
}

// debug
impl Context {
    pub(crate) fn debug_tree(&self) {
        // let mut s = String::new();
        self.print_children_from(NodeId::root());
    }

    pub(crate) fn print_children_from(&self, start: NodeId) {
        eprintln!(" > {start:?}: {:?}", self.get_window_properties().pos());
        if start == NodeId::root() {
            self.recursive_print(None, 0);
        } else {
            self.recursive_print(Some(start), 0);
        }
    }

    fn recursive_print(&self, start: Option<NodeId>, indent: usize) {
        let acc = 3;
        if let Some(current) = start {
            self.tree.get_all_children(&current)
                .map(|children| {
                    children.iter().for_each(|child| {
                        let data = self.get_node_data(child).pos();
                        let name = self.debug_name[child.index()];
                        if self.tree.get_parent(child).is_some_and(|p| self.tree.get_parent(p).is_some()) {
                            for i in 0..(indent - acc)/acc {
                                let c = acc - i;
                                eprint!("{:c$}|", "");
                            }
                            let j = acc - 1;
                            eprintln!("{:j$}╰─ {child:?}: {data:?} | {name:?}", "");
                        } else {
                            eprintln!("{:indent$}╰─ {child:?}: {data:?} | {name:?}", "");
                        }
                        if self.tree.get_first_child(child).is_some() {
                            self.recursive_print(Some(*child), indent + acc);
                        }
                    });
                });
        } else {
            self.tree.get_all_ancestor()
                .iter()
                .for_each(|node| {
                    let data = self.get_node_data(*node).pos();
                    let name = self.debug_name[node.index()];
                    eprintln!(" > {node:?}: {data:?} | {name:?}");
                    if self.tree.get_first_child(node).is_some() {
                        self.recursive_print(Some(**node), indent + acc);
                    }
                });
        }
    }
}

// window
impl Context {
    pub(crate) fn initialize_root(&mut self, size: Size<u32>) {
        self.tree.insert(NodeId::root(), None);
        self.properties.push(Properties::window_properties(size));
        self.debug_name.push(Some("NodeId::ROOT"));
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

// callback
impl Context {
    pub(crate) fn add_style_fn<F: Fn(&mut Properties) + 'static>(&mut self, node_id: NodeId, style_fn: F) {
        self.style_fn.insert(node_id, Box::new(style_fn));
    }

    pub(crate) fn add_callbacks<F: Fn() + 'static>(&mut self, node_id: NodeId, callback: F) {
        self.callbacks.insert(node_id, Box::new(callback));
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
        debug_name: Option<&'static str>,
    ) {
        self.tree.insert(node_id, parent);
        // self.data.insert(&properties);
        self.properties.push(properties);
        self.debug_name.push(debug_name);
    }

    pub(crate) fn add_pixel(&mut self, node_id: NodeId, pixel: Pixel<u8>) {
        let aspect_ratio = pixel.aspect_ratio();
        let texture_id = self.pixels.len();
        let properties = self.get_node_data_mut(&node_id);
        properties.set_texture_id(texture_id as i32);
        properties.adjust_width(aspect_ratio);
        self.pixels.push(pixel);
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
                self.calculate_size(node_id);
            });

        self.recursive_layout(&NodeId::root());
    }

    pub(crate) fn recursive_layout(&mut self, node_id: &NodeId) {
        let children = LayoutContext::new(node_id, self).calculate();

        if node_id != &NodeId::root() {
            self.pending_update.push(*node_id);
        }

        if let Some(children) = children {
            children.iter().for_each(|child| self.recursive_layout(child));
        }
    }

    fn calculate_size(&mut self, node_id: &NodeId) -> Size<u32> {
        let prop = *self.get_node_data(node_id);
        let padding = prop.padding();
        let mut size = prop.size();

        let children = self.tree.get_all_children(node_id);
        if let Some(children) = children {
            children.iter().for_each(|child_id| {
                let child_size = self.calculate_size(child_id);
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

        let final_size = size
            .max(prop.min_width(), prop.min_height())
            .min(prop.max_width(), prop.max_height());

        let properties = self.get_node_data_mut(node_id);
        properties.set_size(final_size);
        final_size
    }
}

// render
impl Context {
    pub(crate) fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub(crate) fn submit_update(&mut self, renderer: &mut Renderer) {
        // let window_size = self.get_window_properties().size();
        // self.pending_update.iter().for_each(|node_id| {
        //     let mut prop = *self.get_node_data(node_id);
        //     if let Some(style_fn) = self.style_fn.get(node_id) {
        //         style_fn(&mut prop);
        //     }
        //     let transform = prop.transform(window_size.into());
        //     if let Some(color) = prop.hover_color() {
        //         renderer
        //             .gfx
        //             .elements
        //             .update(node_id.index() - 1, |elem| elem.set_color(color));
        //     }
        //     renderer.gfx.transforms.update(node_id.index() - 1, |mat| *mat = transform);
        // });
        self.pending_update.clear();
        renderer.update();
    }

    pub(crate) fn detect_hover(&self, cursor: &mut Cursor) {
        // let start = std::time::Instant::now();

        // let hovered = self.properties.iter().enumerate().skip(1).filter(|(_, prop)| {
        //     prop.is_dragable()
        //         || prop.hover_color().is_some()
        //         || prop.click_color().is_some()
        // })
        // .filter(|(_, prop)| prop.is_hovered(cursor))
        // .map(|(idx, _)| idx).min();

        let hovered = self.tree.iter().skip(1).filter_map(|node| {
            let prop = self.get_node_data(node.id());
            if prop.is_hovered(cursor) {
                Some(node.id())
            } else { None }
        }).max();

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

    pub(crate) fn handle_hover(&mut self, cursor: &mut Cursor) {
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(prev_id) = cursor.hover.prev.take() {
            let properties = self.get_node_data(&prev_id);
            if properties.hover_color().is_some() {
                // let idx = prev_id.index();
                // gfx.elements.update(idx - 1, |element| element.set_color(properties.fill_color()));
                self.pending_update.push(prev_id);
            }
        }
        if let Some(hover_id) = cursor.hover.curr.as_ref() {
            // let idx = hover_id.index();
            let properties = self.get_node_data(hover_id);
            let dragable = properties.is_dragable();
            // let hover_color = properties.hover_color();
            // gfx.elements.update(idx - 1, |element| {
            //     if let Some(color) = hover_color {
            //         element.set_color(color);
            //         self.pending_update.push(*hover_id);
            //     }
                if cursor.is_dragging(hover_id) && dragable {
                    self.handle_drag(
                        hover_id,
                        cursor,
                        // &mut gfx.transforms,
                    );
                    self.pending_update.push(*hover_id);
                    eprintln!("updating drag: {hover_id:?}");
                }
            // });
        }
    }

    fn handle_drag(
        &mut self,
        hover_id: &NodeId,
        cursor: &Cursor,
        // transforms: &mut Buffer<Matrix4x4>,
    ) {
        let props = self.get_node_data_mut(hover_id);
        // transforms.update(hover_id.index() - 1, |transform| {
        //     let delta = cursor.hover.pos - cursor.click.offset;
        //     props.adjust_transform(delta, transform);
        // });
        self.recursive_layout(hover_id);
    }

    pub(crate) fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            if let Some(callback) = self.callbacks.get(click_id) {
                callback();
            }
            let idx = click_id.index();
            let props = self.properties[idx];
            cursor.click.offset = cursor.click.pos - Vector2::<f32>::from(props.pos());
            self.pending_update.push(*click_id);
            // gfx.elements.update(idx - 1, |element| {
            //     if let Some(color) = props.click_color() {
            //         element.set_color(color);
            //     }
            // });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = hover_id.index();
                let props = self.properties[idx];
                self.pending_update.push(*hover_id);
                // gfx.elements.update(idx - 1, |element| {
                //     if let Some(color) = props.hover_color() {
                //         element.set_color(color);
                //     }
                // });
            }
        }
    }
}

impl IntoRenderSource for Context {
    type RenderComponentSource = Properties;
    type TetureDataSource = Pixel<u8>;

    fn render_components_source(&self) -> &[Self::RenderComponentSource] { self.properties.as_slice() }

    fn texture_data_source(&self) -> &[Self::TetureDataSource] { self.pixels.as_slice() }
}
