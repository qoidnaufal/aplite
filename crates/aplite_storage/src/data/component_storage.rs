use std::any::TypeId;
use std::ptr::NonNull;
use std::collections::HashMap;

use aplite_bitset::Bitset;

use crate::entity::EntityId;
use crate::map::hash::TypeIdMap;

use crate::data::archetype::{ArchetypeId, ArchetypeTable, ArchetypeBuilder};
use crate::data::component::{ComponentId, Component};
use crate::data::query::{Query, QueryData};

pub struct MarkedBuffer<'a, Q: QueryData> {
    pub(crate) start: NonNull<Q::Item>,
    pub(crate) len: usize,
    marker: std::marker::PhantomData<Q::Fetch<'a>>
}

pub struct MarkedBufferIter<'a, Q: QueryData> {
    start: NonNull<Q::Item>,
    len: usize,
    counter: usize,
    marker: std::marker::PhantomData<Q::Fetch<'a>>,
}

#[derive(Default)]
pub struct ComponentStorage {
    pub(crate) archetype_tables: Vec<ArchetypeTable>,
    pub(crate) archetype_ids: HashMap<Bitset, ArchetypeId>,
    pub(crate) component_ids: TypeIdMap<ComponentId>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            archetype_tables: Vec::new(),
            archetype_ids: HashMap::default(),
            component_ids: TypeIdMap::new(),
        }
    }

    pub(crate) fn get_or_create_id<C: Component + 'static>(&mut self) -> ComponentId {
        if let Some(id) = self.get_component_id::<C>() {
            return id;
        }

        let component_id = ComponentId::new(self.component_ids.len());
        self.component_ids.insert(TypeId::of::<C>(), component_id);

        component_id
    }

    // #[inline(always)]
    // pub(crate) fn insert_from_tuple<C>(&mut self, archetypal_bitset: Bitset, component: C)
    // where
    //     C: Component + 'static
    // {
    //     let component_id = self.component_ids[&TypeId::of::<C>()];
    //     let archetype_id = self.archetype_ids[&archetypal_bitset];
    //     let table = &mut self.archetype_tables[archetype_id.0];
    //     table.insert(component_id, component);
    // }

    pub(crate) fn insert_archetype_by_id<C>(&mut self, archetype_id: ArchetypeId, component: C)
    where
        C: Component + 'static
    {
        let component_id = self.component_ids[&TypeId::of::<C>()];
        let table = &mut self.archetype_tables[archetype_id.0];
        table.insert(component_id, component);
    }

    pub fn insert_component<C: Component>(&mut self, entity: EntityId, component: C) {
        component.insert(entity, self);
    }

    #[inline(always)]
    pub fn get_component_id<C: Component + 'static>(&self) -> Option<ComponentId> {
        self.component_ids.get(&TypeId::of::<C>()).copied()
    }

    pub fn get_archetype_table(&self, bitset: Bitset) -> Option<&ArchetypeTable> {
        self.archetype_ids.get(&bitset).map(|id| &self.archetype_tables[id.0])
    }

    pub fn get_archetype_table_mut(&mut self, bitset: Bitset) -> Option<&mut ArchetypeTable> {
        self.archetype_ids.get(&bitset).map(|id| &mut self.archetype_tables[id.0])
    }

    pub(crate) fn get_archetype_ids<'a>(&'a self, matched_component_ids: Bitset) -> Bitset {
        let bitset_num = self.archetype_ids
            .keys()
            .filter_map(|bits| {
                bits.contains(&matched_component_ids)
                    .then(|| self.archetype_ids.get(bits))?
            })
            .map(|id| 1 << id.0)
            .sum();

        Bitset::new(bitset_num)
    }

    pub fn get_tables<'a>(&'a self, bitset: Bitset) -> Box<[&'a ArchetypeTable]> {
        self.archetype_ids
            .keys()
            .filter_map(|bits| bits.contains(&bitset)
                .then(|| self.archetype_ids.get(bits)
                    .map(|id| &self.archetype_tables[id.0])
                )?
            )
            .collect()
    }

    pub(crate) fn get_marked_buffers<'a, Q>(&'a self, archetype_id_bitset: Bitset) -> Box<[MarkedBuffer<'a, Q>]>
    where
        Q: QueryData,
        Q::Item: Component + 'static,
    {
        let component_id = self.get_component_id::<Q::Item>();

        archetype_id_bitset
            .iter()
            .filter_map(|id| {
                let table = &self.archetype_tables[id as usize];
                let buffer = table.get_component_buffer(component_id?)?;

                Some(MarkedBuffer {
                    start: buffer.raw.block.cast::<Q::Item>(),
                    len: buffer.len(),
                    marker: std::marker::PhantomData,
                })
            })
            .collect()
    }

    pub fn archetype_builder(&mut self) -> ArchetypeBuilder<'_> {
        ArchetypeBuilder {
            storage: self,
            bitset: Bitset::default(),
            table: ArchetypeTable::default(),
        }
    }

    pub fn query<'a, Q>(&'a self) -> Query<'a, Q>
    where
        Q: QueryData,
    {
        Query::new(self)
    }
}

/*
#########################################################
#
# impl MarkedBuffer
#
#########################################################
*/

impl<'a, Q> MarkedBuffer<'a, Q>
where
    Q: QueryData<State = NonNull<<Q as QueryData>::Item>>,
{
    pub fn iter(&'a self) -> impl Iterator<Item = Q::Fetch<'a>> {
        MarkedBufferIter::<Q> {
            start: self.start,
            len: self.len,
            counter: 0,
            marker: std::marker::PhantomData,
        }
    }

    pub fn get(&mut self, offset: usize) -> Option<Q::Fetch<'a>> {
        unsafe {
            if offset < self.len {
                let next = self.start.add(offset);
                return Some(Q::get(next));
            }

            None
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

/*
#########################################################
#
# impl MarkedBufferIter
#
#########################################################
*/

impl<'a, Q> Iterator for MarkedBufferIter<'a, Q>
where
    Q: QueryData<State = NonNull<<Q as QueryData>::Item>>,
{
    type Item = Q::Fetch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.counter < self.len {
                let next = self.start.add(self.counter);
                self.counter += 1;
                return Some(Q::get(next));
            }

            None
        }
    }
}
