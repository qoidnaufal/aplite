use std::collections::HashMap;
use std::any::TypeId;

use crate::entity::Entity;
use crate::map::hash::TypeIdMap;

use super::sparse_index::SparseIndices;
use super::query::{Query, QueryData};
use super::component::{
    Component,
    ComponentBundle,
    ComponentId,
    ComponentBitSet,
    ComponentArray
};

pub struct ComponentTable {
    pub(crate) data_storage: TypeIdMap<ComponentArray>,
    pub(crate) indexes: SparseIndices,
    pub(crate) entities: Vec<Entity>,
    pub(crate) component_id: TypeIdMap<ComponentId>,

    // FIXME: this should be used to store which Vec<Entity>,
    // or archetype are stored with the corresponding bitset
    pub(crate) component_bitset: HashMap<Entity, ComponentBitSet>,

    capacity: usize,
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
            data_storage: TypeIdMap::default(),
            indexes: SparseIndices::default(),
            entities: Vec::with_capacity(capacity),
            component_id: TypeIdMap::default(),
            component_bitset: HashMap::new(),
            capacity,
        }
    }

    pub(crate) fn register_component<T: Component>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<T>();

        if let Some(id) = self.component_id.get(&type_id) {
            return *id;
        }

        let len = self.data_storage.len();
        let component_id = ComponentId(1 << len);

        self.data_storage.insert(type_id, ComponentArray::new::<T>(self.capacity.max(1)));
        self.component_id.insert(type_id, component_id);

        component_id
    }

    pub fn insert<T: Component>(&mut self, entity: &Entity, component: T) {
        let component_id = self.register_component::<T>();

        let type_id = TypeId::of::<T>();
        let array = self.data_storage.get_mut(&type_id).unwrap();
        let len = array.len();

        array.push(component);
        self.indexes.set_index(entity.id(), len);
        self.entities.push(*entity);

        self.component_bitset
            .entry(*entity)
            .or_insert(ComponentBitSet(0))
            .0 |= component_id.0
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
        self.data_storage.clear();
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

        println!("{:#?}", cx.table.component_bitset);

        let ages_array = cx.table.data_storage.get(&std::any::TypeId::of::<Age>()).unwrap();
        let ages_vec = ages_array.iter::<Age>()
            .map(|cell| &unsafe { &*cell.get() }.0)
            .collect::<Vec<_>>();

        println!("{ages_vec:#?}");

        let names_array = cx.table.data_storage.get(&std::any::TypeId::of::<Name>()).unwrap();
        let names_vec = names_array.iter::<Name>()
            .map(|cell| &unsafe { &*cell.get() }.0)
            .collect::<Vec<_>>();

        println!("{names_vec:#?}");

        let location_array = cx.table.data_storage.get(&std::any::TypeId::of::<Location>()).unwrap();
        let location_vec = location_array.iter::<Location>()
            .map(|cell| unsafe {
                let loc = &*cell.get();
                (loc.0, loc.1)
            })
            .collect::<Vec<_>>();

        println!("{location_vec:#?}");

        let indices = &cx.table.indexes.indices;
        println!("{indices:?}");
    }
}
