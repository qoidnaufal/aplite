mod arena;
mod buffer;
mod data;
mod entity;
mod iterator;
mod map;
mod sparse_set;
mod tree;

pub use buffer::TypedErasedBuffer;

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
    non_static_arena::Arena,
    static_arena::StaticArena,
    ptr::{ArenaPtr, ValidCheckedPtr},
};

pub use map::{
    index_map::{IndexMap, Index, IndexMapError},
    hash::{EntityIdMap, TypeIdMap},
};

pub use sparse_set::{
    typed::SparseSet,
    type_erased::TypeErasedSparseSet,
    indices::SparseIndices,
};

pub use iterator::{TreeChildIter, TreeDepthIter};
