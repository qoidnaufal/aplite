use std::collections::HashMap;

use crate::buffer::TypeErasedBuffer;
use crate::data::component::ComponentId;
use crate::entity::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(pub(crate) usize);

impl TableId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

pub(crate) struct TableIndexer {
    table_id: TableId,
    entities: Vec<EntityId>,
}

pub struct ComponentTable {
    pub(crate) components: Vec<TypeErasedBuffer>,
    pub(crate) table_indexer: HashMap<ComponentId, TableIndexer>,
}
