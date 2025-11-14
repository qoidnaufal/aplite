use std::alloc;
use std::cell::UnsafeCell;

use crate::type_erased_array::{UnsafeUntypedArray, Error};
use crate::entity::{EntityId, Entity};

use super::indices::SparseIndices;

pub struct UntypedSparseSet {
    pub(crate) data: UnsafeUntypedArray,
    pub(crate) indexes: SparseIndices,
    pub(crate) entities: Vec<EntityId>,
    item_layout: alloc::Layout,
}

impl Drop for UntypedSparseSet {
    fn drop(&mut self) {
        self.data.clear(self.entities.len(), self.item_layout);
        self.data.dealloc(self.item_layout);
    }
}

impl UntypedSparseSet {
    pub fn with_capacity<T>(capacity: usize) -> Self {
        Self {
            indexes: SparseIndices::default(),
            data: UnsafeUntypedArray::new::<T>(capacity),
            entities: Vec::with_capacity(capacity),
            item_layout: alloc::Layout::new::<T>()
        }
    }

    pub fn get_raw<T>(&self, entity: &Entity) -> Option<*mut u8> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                self.data
                    .get_raw(index.index(), self.item_layout.size())
            })
    }

    pub fn get<T>(&self, entity: &Entity) -> Option<&T> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                &*self.data
                    .get_raw(index.index(), self.item_layout.size())
                    .cast::<T>()
            })
    }

    pub fn get_mut<T>(&self, entity: &Entity) -> Option<&mut T> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                &mut *self.data
                    .get_raw(index.index(), self.item_layout.size())
                    .cast::<T>()
            })
    }

    pub fn get_cell<T>(&self, entity: &Entity) -> Option<&UnsafeCell<T>> {
        self.indexes
            .get_index(entity.id())
            .map(|index| unsafe {
                &*self.data
                    .get_raw(index.index(), self.item_layout.size())
                    .cast::<UnsafeCell<T>>()
            })
    }

    pub fn insert_no_realloc<T>(&mut self, entity: &Entity, value: T) -> Result<(), Error> {
        let size_t = self.item_layout.size();
        if let Some(index) = self.indexes.get_index(entity.id())
            && !index.is_null()
        {
            unsafe {
                let raw = self.data.get_raw(index.index(), size_t);
                std::ptr::write(raw.cast(), value);
                return Ok(())
            }
        }

        let len = self.entities.len();

        if len == self.data.capacity.get() {
            return Err(Error::MaxCapacityReached);
        }

        self.indexes.set_index(entity.id(), len);
        self.data.push(value, len);
        self.entities.push(*entity.id());

        Ok(())
    }

    pub fn insert<T>(&mut self, entity: &Entity, value: T) {
        let size_t = self.item_layout.size();
        if let Some(index) = self.indexes.get_index(entity.id())
            && !index.is_null()
        {
            unsafe {
                let raw = self.data.get_raw(index.index(), size_t);
                return std::ptr::write(raw.cast(), value);
            }
        }

        let len = self.entities.len();

        if len == self.data.capacity.get() {
            self.data.realloc(self.item_layout, len + 1);
        }

        self.indexes.set_index(entity.id(), len);
        self.data.push(value, len);
        self.entities.push(*entity.id());
    }

    pub fn remove<T>(&mut self, entity: Entity) {
        if let Some(index) = self.indexes
            .get_index(&entity.id)
            .map(|i| i.index())
        {
            self.indexes.set_index(self.entities.last().unwrap(), index);
            self.indexes.set_null(&entity.id);
            self.data.swap_remove_drop::<T>(index, self.entities.len() - 1);
            self.entities.swap_remove(index);
        }
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

        let prev_index = set.indexes.get_index(&last.id).copied();

        set.remove::<Obj>(to_remove);
        let removed = set.get::<Obj>(&to_remove);
        assert!(removed.is_none());

        let new_index = set.indexes.get_index(&last.id).copied();
        assert_ne!(prev_index, new_index);

        println!("quitting");

        Ok(())
    }
}
