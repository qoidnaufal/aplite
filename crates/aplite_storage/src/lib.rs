mod arena;
mod buffer;
mod data;
mod entity;
mod iterator;
mod map;
mod sparse_set;
mod tree;

pub use buffer::TypeErasedBuffer;

pub use entity::{
    EntityManager,
    Entity,
    EntityId,
    EntityVersion
};

pub use tree::{
    sparse_tree::{SparseTree, TreeError},
    node::{Node, NodeRef, SubTree},
};

pub use data::{
    component::{Component, ComponentEq, ComponentTuple},
    query::{Query, QueryData, Queryable},
    table::{ArchetypeTable, ComponentStorage},
};

pub use arena::{
    non_static_arena::Arena,
    static_arena::StaticArena,
    ptr::{ArenaPtr, ValidCheckedPtr},
};

pub use map::{
    slot_map::{SlotMap, SlotId, IndexMapError},
    hash::{EntityIdMap, TypeIdMap},
};

pub use sparse_set::{
    SparsetKey,
    typed::SparseSet,
    type_erased::TypeErasedSparseSet,
    indices::SparseIndices,
};

pub use iterator::{TreeChildIter, TreeDepthIter};
