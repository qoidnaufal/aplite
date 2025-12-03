use std::any::TypeId;

use crate::entity::Entity;
use crate::map::hash::TypeIdMap;
use crate::buffer::CpuBuffer;
use crate::sparse_set::indices::SparseIndices;

use super::query::{Query, QueryData};
use super::component::{
    Component,
    ComponentBundle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableId(u64);

impl TableId {
    fn index(&self) -> usize {
        self.0 as _
    }
}

pub struct ComponentTable {
    pub(crate) data: TypeIdMap<CpuBuffer>,
    pub(crate) indexes: SparseIndices,
    pub(crate) entities: Vec<Entity>,
}

impl Default for ComponentTable {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl ComponentTable {
    /// Set the capacity for the contained entity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: TypeIdMap::default(),
            indexes: SparseIndices::default(),
            entities: Vec::with_capacity(capacity),
        }
    }

    pub fn insert<T: Component>(&mut self, entity: &Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let buffer = self.data.get_mut(&type_id).unwrap();
        let len = buffer.len();

        buffer.push(component);
        self.indexes.set_index(entity.id(), len);
        self.entities.push(*entity);
    }

    pub fn insert_bundle<T: ComponentBundle>(&mut self, id: &Entity, bundle: T) {
        bundle.insert_bundle(id, self);
    }

    pub fn query<'a, Q: QueryData<'a>>(&'a self) -> Query<'a, Q> {
        todo!()
    }

    pub fn contains(&self, id: &Entity) -> bool {
        self.entities.contains(id)
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.entities.clear();
        self.indexes.reset();
    }
}

/*
#########################################################
#                                                       #
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod table_test {
    use crate::entity::{EntityManager};
    use super::ComponentTable;
    use crate::data::component::{Component};

    #[derive(Default)]
    struct Context {
        manager: EntityManager,
        table: ComponentTable,
    }

    struct Name(String); impl Component for Name {}
    struct Age(u32); impl Component for Age {}
    struct Location(f32, f32); impl Component for Location {}

    #[test]
    fn insert_get() {
        let mut cx = Context::default();

        for i in 0..5u32 {
            let id = cx.manager.create();
            cx.table.insert_bundle(&id, (Name(i.to_string()), Age(i), Location(i as _, i as _)));
        }

        for i in 0..2u32 {
            let id = cx.manager.create();
            cx.table.insert(&id, Age(i));
        }
    }
}
