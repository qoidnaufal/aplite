mod arena;
mod buffer;
mod data;
mod entity;
mod iterator;
mod map;
mod sparse_set;
mod tree;

pub use buffer::{TypeErasedBuffer, UnmanagedBuffer};

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
    archetype::ArchetypeTable,
    component::{Component, ComponentEq},
    query::{Query, QueryData},
    component_storage::ComponentStorage,
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
    SparseSet,
    indices::{SparseIndices, SparsetKey},
};

pub use iterator::{TreeChildIter, TreeDepthIter};
