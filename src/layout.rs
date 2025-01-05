use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    app::CONTEXT,
    color::Rgb,
    shapes::{Shape, Vertex},
    types::{cast_slice, Size, Vector2},
};

thread_local! {
    pub static NODE_ID: AtomicU64 = const { AtomicU64::new(0) };
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u64);

impl NodeId {
    fn new() -> Self {
        Self(NODE_ID.with(|i| i.fetch_add(1, Ordering::Relaxed)))
    }
}

pub trait Widget: std::fmt::Debug {
    fn id(&self) -> NodeId;
    fn shape(&self) -> Shape;
}

impl Widget for Button {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Button {
    id: NodeId,
}

impl Button {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::new(Vector2::new(), Size::new(1000, 1000), Rgb::RED)
    }
}

#[derive(Debug)]
pub struct Layout {
    pub nodes: Vec<NodeId>,
    pub shapes: HashMap<NodeId, Shape>,
    pub v_offset: HashMap<NodeId, usize>,
    pub i_offset: HashMap<NodeId, usize>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub has_changed: bool,
    last_changed_id: Option<NodeId>,
    used_space: Size<u32>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            v_offset: HashMap::new(),
            i_offset: HashMap::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            used_space: Size::new(0, 0),
            has_changed: false,
            last_changed_id: None,
        }
    }

    pub fn insert(&mut self, node: impl Widget) -> &mut Self {
        let id = node.id();
        let shape = node.shape();
        self.nodes.push(id);
        self.shapes.insert(id, shape);
        self
    }

    pub fn vertices(&self) -> &[u8] {
        cast_slice(&self.vertices).unwrap()
    }

    pub fn indices(&self) -> &[u8] {
        cast_slice(&self.indices).unwrap()
    }

    pub fn indices_len(&self) -> usize {
        self.indices.len()
    }

    pub fn detect_hover(&self) {
        let hovered = self.shapes.iter().find(|(id, shape)| {
            let len = if id.0 < 1 {
                self.i_offset[&NodeId(id.0 + 1)] - self.i_offset[*id]
            } else {
                self.i_offset[*id] - self.i_offset[&NodeId(id.0 - 1)]
            };
            shape.is_hovered(len)
        });
        if let Some((id, _)) = hovered {
            CONTEXT.with_borrow_mut(|ctx| {
                if let Some(click_id) = ctx.cursor.click.obj {
                    ctx.cursor.hover.obj = Some(click_id);
                } else {
                    ctx.cursor.hover.obj = Some(*id);
                }
            })
        } else {
            CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.obj = None)
        }
    }

    pub fn handle_hover(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let Some(ref change_id) = self.last_changed_id.take() {
            if cursor.hover.obj.is_some_and(|hover_id| hover_id != *change_id) || cursor.hover.obj.is_none() {
                let shape = self.shapes.get_mut(change_id).unwrap();
                shape.set_color(|color| *color = Rgb::RED);
                let data = if change_id.0 % 2 == 0 {
                    shape.triangle()
                } else { shape.rectangle() };
                let v_offset = self.v_offset.get(change_id).unwrap();
                self.vertices[*v_offset..v_offset + data.vertices.len()].copy_from_slice(&data.vertices);
                self.has_changed = true;
            }
        }
        if let Some(ref hover_id) = cursor.hover.obj {
            let shape = self.shapes.get_mut(hover_id).unwrap();

            shape.set_color(|color| *color = Rgb::BLUE);
            if cursor.is_dragging(*hover_id) {
                shape.set_color(|color| *color = Rgb::GREEN);
                shape.set_position();
            }
            
            let data = if hover_id.0 % 2 == 0 {
                shape.triangle()
            } else { shape.rectangle() };
            let v_offset = self.v_offset.get(hover_id).unwrap();
            self.vertices[*v_offset..v_offset + data.vertices.len()].copy_from_slice(&data.vertices);
            self.has_changed = true;
            self.last_changed_id = Some(*hover_id);
        }
        
        // self.shapes.iter_mut().for_each(|(id, shape)| {
        //     if cursor.hover.obj.is_some_and(|hover_id| hover_id == *id) {
        //         shape.set_color(|color| *color = Rgb::BLUE);
        //     } else {
        //         shape.set_color(|color| *color = Rgb::RED);
        //     }
        //     let data = if id.0 % 2 == 0 {
        //         shape.triangle()
        //     } else { shape.rectangle() };
        //     let v_offset = self.v_offset.get(id).unwrap();
        //     self.vertices[*v_offset..v_offset + data.vertices.len()].copy_from_slice(&data.vertices);
        //     self.has_changed = true;
        // });
    }

    pub fn handle_click(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let Some(ref click_id) = cursor.click.obj {
            let shape = self.shapes.get_mut(click_id).unwrap();
            shape.set_color(|color| *color = Rgb::GREEN);
            let data = if click_id.0 % 2 == 0 {
                shape.triangle()
            } else { shape.rectangle() };
            let v_offset = self.v_offset.get(click_id).unwrap();
            self.vertices[*v_offset..v_offset + data.vertices.len()].copy_from_slice(&data.vertices);
            self.has_changed = true;
            self.last_changed_id = Some(*click_id)
        }
        
        // self.shapes.iter_mut().for_each(|(id, shape)| {
        //     if cursor.hover.obj.is_some_and(|hover_id| hover_id == *id)
        //         && cursor.click.obj.is_some_and(|click_id| click_id == *id) {
        //         shape.set_color(|color| *color = Rgb::GREEN);
        //     } else {
        //         shape.set_color(|color| *color = Rgb::RED);
        //     }
        //     let data = if id.0 % 2 == 0 {
        //         shape.triangle()
        //     } else { shape.rectangle() };
        //     let v_offset = self.v_offset.get(id).unwrap();
        //     self.vertices[*v_offset..v_offset + data.vertices.len()].copy_from_slice(&data.vertices);
        //     self.has_changed = true;
        // });
    }

    pub fn calculate(&mut self) {
        let mut indices_offset = 0;
        let mut vertices_offset = 0;

        self.nodes.iter().for_each(|id| {
            if let Some(shape) = self.shapes.get_mut(id) {
                shape.pos.y += self.used_space.height;
                let mut data = if id.0 % 2 == 0 {
                    shape.triangle()
                } else { shape.rectangle() };

                data.indices.iter_mut().for_each(|idx| *idx += indices_offset as u32);

                self.v_offset.insert(*id, vertices_offset);
                vertices_offset += data.vertices.len();

                self.i_offset.insert(*id, indices_offset);
                indices_offset += data.indices.len();

                self.vertices.extend_from_slice(&data.vertices);
                self.indices.extend_from_slice(&data.indices);

                self.used_space.height += shape.size.height;
            }
        });
    }
}

