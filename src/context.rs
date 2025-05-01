use util::{Matrix4x4, Size, Vector2};

use crate::color::Pixel;
use crate::layout::Layout;
use crate::cursor::{Cursor, MouseAction};
use crate::renderer::{Buffer, Element, Gfx, IntoRenderSource, Renderer};
use crate::properties::{Orientation, Properties};
use crate::callback::CALLBACKS;
use crate::tree::{NodeId, Tree};

pub struct Context {
    pub(crate) tree: Tree<NodeId>,
    properties: Vec<Properties>,
    pixels: Vec<Pixel<u8>>,
    layout: Layout,
    pending_update: Vec<NodeId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            tree: Default::default(),
            properties: Vec::with_capacity(1024),
            pixels: Vec::new(),
            layout: Layout::new(),
            pending_update: Vec::with_capacity(10),
        }
    }
}

impl Context {
    #[allow(unused)]
    pub(crate) fn print_nodes(&self) {
        let to_print = self
            .tree
            .into_iter()
            .map(|node| (node.id(), node.parent(), self.tree.get_all_children(node.id())))
            .collect::<Vec<_>>();
        for (id, parent, children) in to_print {
            eprintln!("{id:?} | {parent:?} | {children:?}");
        }
    }
}

impl Context {
    pub(crate) fn create_entity(&self) -> NodeId {
        self.tree.create_entity()
    }

    pub(crate) fn initialize_root(&mut self, window: &winit::window::Window) {
        self.tree.insert(NodeId::root(), None);
        self.properties.push(Properties::window_properties(window));
    }

    pub(crate) fn insert(&mut self,
        node_id: NodeId,
        maybe_parent: Option<NodeId>,
        mut properties: Properties,
        maybe_pixel: Option<Pixel<u8>>,
    ) {
        self.tree.insert(node_id, maybe_parent);
        if let Some(pixel) = maybe_pixel {
            let texture_id = self.pixels.len() as i32;
            let aspect_ratio = pixel.aspect_ratio();
            self.pixels.push(pixel);
            properties.set_texture_id(texture_id);
            properties.adjust_ratio(aspect_ratio);
        }
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
        let idx = self.tree.iter().position(|node| node.id() == node_id).unwrap();
        &self.properties[idx]
    }

    pub(crate) fn get_node_data_mut(&mut self, node_id: &NodeId) -> &mut Properties {
        let idx = self.tree.iter().position(|node| node.id() == node_id).unwrap();
        &mut self.properties[idx]
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
        let hovered = self.tree.into_iter().filter_map(|node| {
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
            let idx = self.tree.into_iter().position(|node| *node.id() == prev_id).unwrap();
            let properties = self.get_node_data(&prev_id);
            gfx.elements.update(idx, |element| element.set_color(properties.fill_color()));
            self.pending_update.push(prev_id);
        }
        if let Some(hover_id) = cursor.hover.curr.as_ref() {
            let idx = self.tree.into_iter().position(|node| node.id() == hover_id).unwrap();
            let properties = self.get_node_data(hover_id);
            let dragable = properties.is_dragable();
            let hover_color = properties.hover_color();
            gfx.elements.update(idx, |element| {
                if let Some(color) = hover_color {
                    element.set_color(color);
                }
                if cursor.is_dragging(hover_id) && dragable {
                    self.handle_drag(
                        &hover_id,
                        cursor,
                        element,
                        &mut gfx.transforms,
                    );
                }
            });
            self.pending_update.push(*hover_id);
        }
    }

    fn handle_drag(
        &mut self,
        hover_id: &NodeId,
        cursor: &Cursor,
        element: &mut Element,
        transforms: &mut Buffer<Matrix4x4>,
    ) {
        let props = self.get_node_data_mut(hover_id);
        transforms.update(element.transform_id as usize, |transform| {
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
        let children = self
            .tree
            .get_all_children(node_id)
            .map(|nodes| {
                nodes
                    .iter()
                    .map(|n| **n)
                    .collect::<Vec<_>>()
            });

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
                let child_idx = self.tree.into_iter().position(|node| node.id() == child_id).unwrap();
                transforms.update(child_idx, |child_transform| {
                    self.assign_position(child_id);
                    let child_props = self.properties[child_idx];
                    let x = child_props.pos().x as f32 / (child_props.size().width as f32 / child_transform[0].x) * 2.0 - 1.0;
                    let y = 1.0 - child_props.pos().y as f32 / (child_props.size().height as f32 / child_transform[1].y) * 2.0;
                    child_transform.translate(x, y);
                });

                self.handle_child_relayout(child_id, transforms);
            });

            if let Some(parent_id) = self.tree.get_parent(&node_id).cloned() {
                self.reset_to_parent(&parent_id, pos, size / 2);
            }
        }
    }

    pub(crate) fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            CALLBACKS.with_borrow(|cb| cb.run(click_id));
            let idx = self.tree.into_iter().position(|node| node.id() == click_id).unwrap();
            let props = self.properties[idx];
            cursor.click.offset = cursor.click.pos - Vector2::<f32>::from(props.pos());
            gfx.elements.update(idx, |element| {
                if let Some(color) = props.click_color() {
                    element.set_color(color);
                    self.pending_update.push(*click_id);
                }
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = self.tree.into_iter().position(|node| node.id() == hover_id).unwrap();
                let props = self.properties[idx];
                gfx.elements.update(idx, |element| {
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

    fn components(&self) -> &[Self::RC] { self.properties.as_slice() }

    fn textures(&self) -> &[Self::TD] { self.pixels.as_slice() }
}
