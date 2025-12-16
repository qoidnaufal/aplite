mod arena;
mod buffer;
mod data;
mod entity;
mod iterator;
mod map;
mod sparse_set;
mod tree;

pub use buffer::CpuBuffer;
pub use entity::{
    EntityManager,
    Entity,
    EntityId,
    EntityVersion
};

pub use tree::{
    tree::{Tree, TreeError},
    node::{Node, NodeRef, SubTree},
};

pub use data::{
    component::Component,
    query::Query,
    table::{ComponentTable, ComponentStorage, ComponentRegistrator},
};

pub use arena::{
    typed::TypedArena,
    untyped::Arena,
    ptr::Ptr,
};

pub use map::{
    index_map::{IndexMap, Index, IndexMapError},
    hash::{EntityIdMap, TypeIdMap},
};

pub use sparse_set::{
    typed::SparseSet,
    untyped::UntypedSparseSet,
    indices::SparseIndices,
};

pub use iterator::{TreeChildIter, TreeDepthIter};
