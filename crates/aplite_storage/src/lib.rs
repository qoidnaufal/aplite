mod arena;
mod data;
mod entity;
mod iterator;
mod map;
mod sparse_set;
mod tree;
mod type_erased_array;

pub use entity::{EntityManager, Entity, EntityId, EntityVersion};
pub use type_erased_array::UntypedArray;
pub use tree::{
    tree::{Tree, TreeError},
    node::{Node, NodeRef, SubTree},
};
pub use data::{
    component::{Component, IntoComponent},
    query::Query,
    table::ComponentTable,
};

pub use arena::{
    typed::TypedArena,
    untyped::Arena,
    item::ArenaItem,
};
pub use map::{
    index_map::{IndexMap, Index, IndexMapError},
    hash::EntityIdMap,
};
pub use sparse_set::{
    typed::SparseSet,
    untyped::UntypedSparseSet,
    indices::SparseIndices,
};

pub use iterator::{TreeChildIter, TreeDepthIter};
