use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicU64, Ordering},
    cell::RefCell,
};

use crate::{
    color::Rgb,
    shapes::Shape,
    types::{cast_slice, Size, Vector3},
};

thread_local! {
    pub static NODE_ID: AtomicU64 = AtomicU64::new(0);
    pub static NODE_TREE: RefCell<NodeTree> = RefCell::new(NodeTree::new());
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u64);

impl NodeId {
    fn new() -> Self {
        Self(NODE_ID.with(|i| i.fetch_add(1, Ordering::Relaxed)))
    }
}

#[derive(Debug, Default)]
pub struct NodeTree {
    root: NodeId,
    nodes: HashSet<NodeId>,
    children: HashMap<NodeId, Vec<NodeId>>,
}

impl NodeTree {
    fn new() -> Self {
        Self {
            root: NodeId::new(),
            nodes: HashSet::new(),
            children: HashMap::new()
        }
    }

    fn push(&mut self, id: NodeId) {
        self.nodes.insert(id);
    }
}

#[derive(Debug)]
pub struct Button {
    id: NodeId,
}

impl Button {
    pub fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> &NodeId {
        &self.id
    }

    fn shape(&self) -> Shape {
        if self.id().0 % 2 == 0 {
            Shape::triangle(Vector3::new(), Size::new(1000, 1000), Rgb::RED)
        } else {
            Shape::rectangle(Vector3::new(), Size::new(1000, 1000), Rgb::RED)
        }
    }

    fn data(&self) -> Vec<u8> {
        let vertices = self.shape().vertices;
        let data = cast_slice(&vertices).unwrap();
        data.to_vec()
    }
}

#[derive(Debug)]
pub struct Layout {
    pub nodes: Vec<Button>,
    pub shapes: HashMap<NodeId, Shape>,
    pub indices: Vec<u32>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            indices: Vec::new(),
        }
    }

    pub fn insert(&mut self, node: Button) -> &mut Self {
        self.nodes.push(node);
        self
    }

    pub fn vertices(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.nodes.iter().for_each(|node| {
            let id = node.id();
            let shape = self.shapes.get(id).unwrap();
            let data: &[u8] = cast_slice(&shape.vertices).unwrap();
            buf.extend_from_slice(data);
        });
        buf
    }

    pub fn indices(&self) -> &[u8] {
        cast_slice(&self.indices).unwrap()
    }

    pub fn handle_click(&mut self) {
        self.shapes.iter_mut().for_each(|(_, shape)| {
            shape.handle_click();
        });
    }

    pub fn set_position(&mut self) {
        self.shapes.iter_mut().for_each(|(_, shape)| {
            shape.set_position();
        });
    }

    pub fn calculate(&mut self) {
        let mut height_offset = 0.0;
        let mut indices_offset = 0;
        let mut margin = 0.0;

        self.nodes.iter_mut().for_each(|node| {
            let mut shape = node.shape();
            shape.vertices.iter_mut().for_each(|vert| {
                vert.position.y += height_offset + margin;
            });
            height_offset += shape.dimension().height;
        
            shape.indices.iter_mut().for_each(|idx| *idx += indices_offset as u32);
            indices_offset += shape.indices.len();
            println!("{:?}", shape);

            self.indices.extend_from_slice(&shape.indices);
            self.shapes.insert(node.id, shape);

            margin -= 0.03;
        });
    }
}

