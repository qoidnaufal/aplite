use std::alloc;
use std::cell::UnsafeCell;

use crate::buffer::{RawCpuBuffer, Error};
use crate::entity::{EntityId, Entity};

use super::indices::SparseIndices;

pub struct UntypedSparseSet {
    pub(crate) data: RawCpuBuffer,
    pub(crate) indexes: SparseIndices,
    pub(crate) entities: Vec<EntityId>,
    item_layout: alloc::Layout,
}

impl Drop for UntypedSparseSet {
    fn drop(&mut self) {
        self.clear();
        self.data.dealloc(self.item_layout);
    }
}

impl UntypedSparseSet {
    pub fn new<T>() -> Self {
        Self::with_capacity::<T>(0)
    }

    #[inline(always)]
    pub fn with_capacity<T>(capacity: usize) -> Self {
        let item_layout = alloc::Layout::new::<T>();

        Self {
            indexes: SparseIndices::default(),
            data: RawCpuBuffer::with_capacity::<T>(capacity, item_layout),
            entities: Vec::with_capacity(capacity),
            item_layout
        }
    }

    #[inline(always)]
    pub fn get_raw<T>(&self, entity: &Entity) -> Option<*mut u8> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                self.data
                    .get_raw(index * self.item_layout.size())
            })
    }

    #[inline(always)]
    pub fn get<T>(&self, entity: &Entity) -> Option<&T> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                &*self.data
                    .get_raw(index * self.item_layout.size())
                    .cast::<T>()
            })
    }

    #[inline(always)]
    pub fn get_mut<T>(&self, entity: &Entity) -> Option<&mut T> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                &mut *self.data
                    .get_raw(index * self.item_layout.size())
                    .cast::<T>()
            })
    }

    pub fn get_cell<T>(&self, entity: &Entity) -> Option<&UnsafeCell<T>> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                &*self.data
                    .get_raw(index * self.item_layout.size())
                    .cast::<UnsafeCell<T>>()
            })
    }

    pub fn insert_no_realloc<T>(&mut self, entity: &Entity, value: T) -> Result<(), Error> {
        if let Some(exist) = self.get_mut(entity) {
            *exist = value;
            return Ok(());
        }

        let len = self.entities.len();
        if len == self.data.capacity {
            return Err(Error::MaxCapacityReached);
        }

        self.insert_inner(entity, len, value);

        Ok(())
    }

    pub fn insert<T>(&mut self, entity: &Entity, value: T) {
        if let Some(exist) = self.get_mut(entity) {
            *exist = value;
            return;
        }

        let len = self.entities.len();
        if len == self.data.capacity {
            self.data.realloc_unmanaged(self.item_layout, len + 4);
        }

        self.insert_inner(entity, len, value);
    }

    #[inline(always)]
    fn insert_inner<T>(&mut self, entity: &Entity, len: usize, value: T) {
        self.indexes.set_index(entity.id(), len);
        self.data.push_unmanaged(value, len);
        self.entities.push(*entity.id());
    }

    pub fn remove<T>(&mut self, entity: Entity) {
        if let Some(index) = self.indexes.get_index(&entity.id) {
            self.indexes.set_index(self.entities.last().unwrap(), index);
            self.indexes.set_null(&entity.id);
            unsafe {
                let ptr = self.data
                    .swap_remove_unmanaged::<T>(index, self.entities.len() - 1);
                std::ptr::drop_in_place(ptr);
            }
            self.entities.swap_remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.indexes.reset();
        self.data.clear(self.entities.len());
        self.entities.clear();
    }
}

#[cfg(test)]
mod untyped_sparse_set_test {
    use super::*;

    #[derive(Debug)]
    struct Obj {
        name: String,
        age: u32,
    }

    impl Drop for Obj {
        fn drop(&mut self) {
            println!("dropped {} aged {}", self.name, self.age)
        }
    }

    impl Obj {
        fn new(name: &str, age: u32) -> Self {
            Self {
                name: name.to_string(),
                age,
            }
        }
    }

    #[test]
    fn get() -> Result<(), Error> {
        let entity = Entity::new(0, 0);
        let mut set = UntypedSparseSet::with_capacity::<Obj>(1);
        set.insert_no_realloc(&entity, Obj::new("Balo", 69))?;

        let balo = set.get::<Obj>(&entity);
        println!("{balo:?}");

        let cell = set.get_cell::<Obj>(&entity);
        let obj = cell.map(|cell| unsafe { &*cell.get() });
        println!("{obj:?}");

        Ok(())
    }

    #[test]
    fn swap_remove() -> Result<(), Error> {
        const NUM: usize = 5;
        let mut set = UntypedSparseSet::with_capacity::<Obj>(NUM);
        let names = ["Balo", "Nunez", "Maguirre", "Bendtner", "Haryono"];

        for i in 0..NUM {
            let obj = Obj::new(names[i], i as _);
            set.insert_no_realloc(&Entity::new(i as _, 0), obj)?;
        }

        let last = Entity::new(4, 0);
        let to_remove = Entity::new(1, 0);

        let prev_index = set.indexes.get_index(&last.id);

        set.remove::<Obj>(to_remove);
        let removed = set.get::<Obj>(&to_remove);
        assert!(removed.is_none());

        let new_index = set.indexes.get_index(&last.id);
        assert_ne!(prev_index, new_index);

        println!("quitting");

        Ok(())
    }
}
