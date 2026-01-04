use std::collections::HashMap;

use crate::data::component::ComponentId;
use crate::entity::EntityId;
use crate::sparse_set::{SparseSet, TypeErasedSparseSet};
use crate::buffer::RawBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(pub(crate) usize);

impl TableId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

pub(crate) struct Table {
    data: TypeErasedSparseSet<EntityId>,
}

pub struct ComponentTable {
    pub(crate) components: SparseSet<ComponentId, RawBuffer>,
    pub(crate) table_indexer: HashMap<ComponentId, TableId>,
}
