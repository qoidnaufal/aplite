use std::collections::HashMap;

use util::{Matrix4x4, Size, Vector2};

use crate::color::Pixel;
use crate::layout::Layout;
use crate::cursor::{Cursor, MouseAction};
use crate::renderer::{Buffer, Gfx, IntoRenderSource, Renderer};
use crate::properties::{Orientation, Properties};
use crate::tree::{Entity, NodeId, Tree};

pub struct Context {
    current: NodeId,
    pub(crate) tree: Tree<NodeId>,
    properties: Vec<Properties>,
    pixels: Vec<Pixel<u8>>,
    layout: Layout,
    style_fn: HashMap<NodeId, Box<dyn Fn(&mut Properties)>>,
    callbacks: HashMap<NodeId, Box<dyn Fn()>>,
    pending_update: Vec<NodeId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            current: NodeId::root(),
            tree: Default::default(),
            properties: Vec::with_capacity(1024),
            pixels: Vec::new(),
            layout: Layout::new(),
            style_fn: HashMap::new(),
            callbacks: HashMap::new(),
            pending_update: Vec::with_capacity(10),
        }
    }
}

impl Context {
    #[allow(unused)]
    pub(crate) fn print_nodes(&self, start: Option<NodeId>, indent: usize) {
        let acc = 3;
        if let Some(current) = start {
            self
                .tree
                .iter()
                .filter_map(|node| {
                    if node.id() == &current {
                        self.tree.get_all_children(node.id())
                    } else { None }
                })
                .for_each(|children| {
                    children.iter().for_each(|child| {
                        if self.tree.get_parent(child).is_some_and(|p| self.tree.get_parent(p).is_some()) {
                            for i in 0..(indent - acc)/acc {
                                let c = acc - i;
                                eprint!("{:c$}|", "");
                            }
                            let j = acc - 1;
                            eprintln!("{:j$}╰─ {child:?}", "");
                        } else {
                            eprintln!("{:indent$}╰─ {child:?}", "");
                        }
                        if self.tree.get_first_child(child).is_some() {
                            self.print_nodes(Some(*child), indent + acc);
                        }
                    });
                });
        } else {
            self
                .tree
                .iter()
                .filter(|node| node.parent().is_none())
                .for_each(|node| {
                    eprintln!(" - {:?}", node.id());
                    if node.first_child().is_some() {
                        self.print_nodes(Some(*node.id()), indent + acc);
                    }
                });
        }
    }

    pub(crate) fn add_style_fn<F: Fn(&mut Properties) + 'static>(&mut self, node_id: NodeId, style_fn: F) {
        self.style_fn.insert(node_id, Box::new(style_fn));
    }

    pub(crate) fn add_callbacks<F: Fn() + 'static>(&mut self, node_id: NodeId, callback: F) {
        self.callbacks.insert(node_id, Box::new(callback));
    }

    pub(crate) fn add_pixel(&mut self, node_id: NodeId, pixel: Pixel<u8>) {
        let texture_id = self.pixels.len();
        let properties = self.get_node_data_mut(&node_id);
        properties.set_texture_id(texture_id as i32);
        self.pixels.push(pixel);
    }
}

impl Context {
    pub(crate) fn create_entity(&self) -> NodeId {
        self.tree.create_entity()
    }

    pub(crate) fn current_entity(&self) -> Option<NodeId> {
        if self.current == NodeId::root() {
            None
        } else {
            Some(self.current)
        }
    }

    pub(crate) fn set_current_entity(&mut self, maybe_entity: Option<NodeId>) {
        if let Some(entity) = maybe_entity {
            self.current = entity;
        } else {
            self.current = NodeId::root();
        }
    }

    pub(crate) fn initialize_root(&mut self, size: Size<u32>) {
        self.tree.insert(NodeId::root(), None);
        self.properties.push(Properties::window_properties(size));
    }

    pub(crate) fn insert(
        &mut self,
        node_id: NodeId,
        maybe_parent: Option<NodeId>,
        properties: Properties,
    ) {
        self.tree.insert(node_id, maybe_parent);
        self.properties.push(properties);
    }

    pub(crate) fn get_window_properties(&self) -> &Properties {
        &self.properties[0]
    }

    pub(crate) fn update_window_properties<F: Fn(&mut Properties)>(&mut self, f: F) {
        if let Some(prop) = self.properties.get_mut(0) {
            f(prop);
        }
    }

    pub(crate) fn get_node_data(&self, node_id: &NodeId) -> &Properties {
        &self.properties[node_id.index()]
    }

    pub(crate) fn get_node_data_mut(&mut self, node_id: &NodeId) -> &mut Properties {
        &mut self.properties[node_id.index()]
    }

    pub(crate) fn set_orientation(&mut self, node_id: &NodeId) {
        let properties = self.get_node_data(node_id);
        self.layout.set_orientation(properties.orientation());
    }

    pub(crate) fn set_alignment(&mut self, node_id: &NodeId) {
        let properties = self.get_node_data(node_id);
        self.layout.set_alignment(properties.alignment());
    }

    pub(crate) fn set_spacing(&mut self, node_id: &NodeId) {
        let properties = self.get_node_data(node_id);
        self.layout.set_spacing(properties.spacing());
    }

    pub(crate) fn set_padding(&mut self, node_id: &NodeId) {
        let properties = self.get_node_data(node_id);
        self.layout.set_padding(properties.padding());
    }

    pub(crate) fn set_next_pos(&mut self, f: impl FnOnce(&mut Vector2<u32>)) {
        self.layout.set_next_pos(f);
    }

    pub(crate) fn assign_position(&mut self, node_id: &NodeId) {
        let next_pos = self.layout.next_pos();
        let properties = self.get_node_data_mut(node_id);
        let half = properties.size() / 2;
        properties.set_position(next_pos + half);
        let pos = properties.pos();
        let size = properties.size();

        self.layout.assign_position(pos, size);
    }

    pub(crate) fn reset_to_parent(
        &mut self,
        node_id: &NodeId,
        current_pos: Vector2<u32>,
        half: Size<u32>
    ) {
        self.set_orientation(node_id);
        self.set_alignment(node_id);
        self.set_spacing(node_id);
        self.set_padding(node_id);
        self.layout.reset_to_parent(current_pos, half);
    }

    pub(crate) fn calculate_size(&mut self, node_id: &NodeId) {
        let prop = *self.get_node_data(node_id);
        let padding = prop.padding();
        let mut size = prop.size();

        let children = self.tree.get_all_children(node_id);
        if let Some(children) = children {
            children.iter().for_each(|child_id| {
                self.calculate_size(child_id);
                let child_size = self.get_node_data(child_id).size();
                match prop.orientation() {
                    Orientation::Vertical => {
                        size.height += child_size.height;
                        size.width = size.width.max(child_size.width + padding.horizontal());
                    }
                    Orientation::Horizontal => {
                        size.height = size.height.max(child_size.height + padding.vertical());
                        size.width += child_size.width - 1;
                    }
                }
            });
            let child_len = children.len() as u32;
            let stretch = prop.spacing() * (child_len - 1);
            match prop.orientation() {
                Orientation::Vertical => {
                    size.height += padding.vertical() + stretch;
                },
                Orientation::Horizontal => {
                    size.width += padding.horizontal() + stretch;
                },
            }
        }

        let final_size = size
            .max(prop.min_width(), prop.min_height())
            .min(prop.max_width(), prop.max_height());

        let properties = self.get_node_data_mut(node_id);
        properties.set_size(final_size);
    }

    pub(crate) fn layout(&mut self) {}
}

// .....................................................................

impl Context {
    pub(crate) fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub(crate) fn submit_update(&mut self, renderer: &mut Renderer) {
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

    pub(crate) fn handle_hover(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(prev_id) = cursor.hover.prev.take() {
            let properties = self.get_node_data(&prev_id);
            if properties.hover_color().is_some() {
                let idx = prev_id.index();
                gfx.elements.update(idx - 1, |element| element.set_color(properties.fill_color()));
                self.pending_update.push(prev_id);
            }
        }
        if let Some(hover_id) = cursor.hover.curr.as_ref() {
            let idx = hover_id.index();
            let properties = self.get_node_data(hover_id);
            let dragable = properties.is_dragable();
            let hover_color = properties.hover_color();
            gfx.elements.update(idx - 1, |element| {
                if let Some(color) = hover_color {
                    element.set_color(color);
                    self.pending_update.push(*hover_id);
                }
                if cursor.is_dragging(hover_id) && dragable {
                    self.handle_drag(
                        hover_id,
                        cursor,
                        &mut gfx.transforms,
                    );
                    self.pending_update.push(*hover_id);
                }
            });
        }
    }

    fn handle_drag(
        &mut self,
        hover_id: &NodeId,
        cursor: &Cursor,
        transforms: &mut Buffer<Matrix4x4>,
    ) {
        let props = self.get_node_data_mut(hover_id);
        transforms.update(hover_id.index() - 1, |transform| {
            let delta = cursor.hover.pos - cursor.click.offset;
            props.set_transform(delta, transform);
        });
        self.handle_child_relayout(hover_id, transforms);
    }

    fn handle_child_relayout(
        &mut self,
        node_id: &NodeId,
        transforms: &mut Buffer<Matrix4x4>,
    ) {
        let children = self.tree.get_all_children(node_id);

        if let Some(children) = children {
            self.set_orientation(node_id);
            self.set_spacing(node_id);
            self.set_padding(node_id);
            let properties = self.get_node_data(node_id);
            let pos = properties.pos();
            let size = properties.size();
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

            children.iter().for_each(|child_id| {
                let child_idx = child_id.index();
                transforms.update(child_idx - 1, |child_transform| {
                    self.assign_position(child_id);
                    let child_props = self.properties[child_idx];
                    let x = child_props.pos().x as f32 / (child_props.size().width as f32 / child_transform[0].x) * 2.0 - 1.0;
                    let y = 1.0 - child_props.pos().y as f32 / (child_props.size().height as f32 / child_transform[1].y) * 2.0;
                    child_transform.translate(x, y);
                });

                self.handle_child_relayout(child_id, transforms);
            });

            if let Some(parent_id) = self.tree.get_parent(node_id).cloned() {
                self.reset_to_parent(&parent_id, pos, size / 2);
            }
        }
    }

    pub(crate) fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            if let Some(callback) = self.callbacks.get(click_id) {
                callback();
            }
            let idx = click_id.index();
            let props = self.properties[idx];
            cursor.click.offset = cursor.click.pos - Vector2::<f32>::from(props.pos());
            gfx.elements.update(idx - 1, |element| {
                if let Some(color) = props.click_color() {
                    element.set_color(color);
                    self.pending_update.push(*click_id);
                }
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = hover_id.index();
                let props = self.properties[idx];
                gfx.elements.update(idx - 1, |element| {
                    if let Some(color) = props.hover_color() {
                        element.set_color(color);
                        self.pending_update.push(*hover_id);
                    }
                });
            }
        }
    }
}

impl IntoRenderSource for Context {
    type RC = Properties;
    type TD = Pixel<u8>;

    fn render_components_source(&self) -> &[Self::RC] { self.properties.as_slice() }

    fn texture_data_source(&self) -> &[Self::TD] { self.pixels.as_slice() }
}
