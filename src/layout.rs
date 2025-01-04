use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    app::{MouseAction, MouseButton, CONTEXT}, color::Rgb, shapes::{Shape, Vertex}, types::{cast_slice, Size, Vector2}
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
    pub vertices: HashMap<NodeId, Vec<Vertex>>,
    pub indices: HashMap<NodeId, Vec<u32>>,
    used_space: Size<u32>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            vertices: HashMap::new(),
            indices: HashMap::new(),
            used_space: Size::new(0, 0),
        }
    }

    pub fn insert(&mut self, node: impl Widget) -> &mut Self {
        let id = node.id();
        let shape = node.shape();
        self.nodes.push(id);
        self.shapes.insert(id, shape);
        self
    }

    pub fn vertices(&self) -> Vec<u8> {
        let mut vertices = Vec::new();
        self.nodes.iter().for_each(|id| {
            let vert = self.vertices.get(id).unwrap();
            vertices.extend_from_slice(cast_slice(vert).unwrap());
        });
        vertices
    }

    pub fn indices(&self) -> Vec<u8> {
        let mut indices = Vec::new();
        self.nodes.iter().for_each(|id| {
            let idx = self.indices.get(id).unwrap();
            indices.extend_from_slice(cast_slice(idx).unwrap());
        });
        indices
    }

    pub fn indices_len(&self) -> usize {
        self.nodes.iter().map(|id| {
            let idx = self.indices.get(id).unwrap();
            idx.len()
        }).sum()
    }

    pub fn detect_hover(&self) {
        let hovered = self.shapes.iter().find(|(id, shape)| {
            if let Some(indices) = self.indices.get(id) {
                let len = indices.len();
                shape.is_hovered(len)
            } else { false }
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

    pub fn detect_click(&self) {
        CONTEXT.with_borrow_mut(|ctx| {
            match (ctx.cursor.state.action, ctx.cursor.state.button) {
                (MouseAction::Pressed, MouseButton::Left) => {
                    ctx.cursor.click.obj = ctx.cursor.hover.obj;
                    ctx.cursor.click.pos = ctx.cursor.hover.pos;
                },
                (MouseAction::Released, MouseButton::Left) => ctx.cursor.click.obj = None,
                _ => {}
            }
        })
    }

    pub fn handle_hover(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        self.shapes.iter_mut().zip(&mut self.vertices).for_each(|((id, shape), (_, vert))| {
            if cursor.hover.obj.is_some_and(|hover_id| hover_id == *id) {
                shape.set_color(|color| *color = Rgb::BLUE);
            } else {
                shape.set_color(|color| *color = Rgb::RED);
            }
            let data = if id.0 % 2 == 0 {
                shape.triangle()
            } else { shape.rectangle() };
            *vert = data.vertices;
        });
    }

    pub fn handle_click(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        self.shapes.iter_mut().zip(&mut self.vertices).for_each(|((id, shape), (_, vert))| {
            if cursor.hover.obj.is_some_and(|hover_id| hover_id == *id)
                && cursor.click.obj.is_some_and(|click_id| click_id == *id) {
                shape.set_color(|color| *color = Rgb::GREEN);
            } else {
                shape.set_color(|color| *color = Rgb::RED);
            }
            let data = if id.0 % 2 == 0 {
                shape.triangle()
            } else { shape.rectangle() };
            *vert = data.vertices;
        });
    }

    pub fn handle_drag(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        self.shapes.iter_mut().zip(&mut self.vertices).for_each(|((id, shape), (_, vert))| {
            if cursor.hover.obj.is_some_and(|hover_id| hover_id == *id)
                && cursor.click.obj.is_some_and(|click_id| click_id == *id) {
                shape.set_color(|color| *color = Rgb::GREEN);
                shape.set_position();
            }
            let data = if id.0 % 2 == 0 {
                shape.triangle()
            } else { shape.rectangle() };
            *vert = data.vertices;
        });
    }

    pub fn calculate(&mut self) {
        let mut indices_offset = 0;

        self.nodes.iter().for_each(|id| {
            if let Some(shape) = self.shapes.get_mut(id) {
                shape.pos.y += self.used_space.height;
                let mut data = if id.0 % 2 == 0 {
                    shape.triangle()
                } else { shape.rectangle() };

                data.indices.iter_mut().for_each(|idx| *idx += indices_offset as u32);

                self.used_space.height += shape.size.height;
                indices_offset += data.indices.len();

                self.indices.insert(*id, data.indices.clone());
                self.vertices.insert(*id, data.vertices);
            }
        });
    }
}

