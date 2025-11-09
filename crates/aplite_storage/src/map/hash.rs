use std::hash::{Hasher, BuildHasher};
use std::any::TypeId;
use std::collections::HashMap;

pub(crate) type TypeIdMap<V> = HashMap<TypeId, V, NullHashBuilder>;

pub(crate) struct NullHash(u64);

impl Hasher for NullHash {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("Should never be called")
    }

    fn write_u8(&mut self, i: u8) { self.0 = i as _ }
    fn write_i8(&mut self, i: i8) { self.0 = i as _ }

    fn write_u16(&mut self, i: u16) { self.0 = i as _ }
    fn write_i16(&mut self, i: i16) { self.0 = i as _ }

    fn write_u32(&mut self, i: u32) { self.0 = i as _ }
    fn write_i32(&mut self, i: i32) { self.0 = i as _ }

    fn write_u64(&mut self, i: u64) { self.0 = i }
    fn write_i64(&mut self, i: i64) { self.0 = i as _ }

    fn write_u128(&mut self, i: u128) { self.0 = i as _ }
    fn write_i128(&mut self, i: i128) { self.0 = i as _ }

    fn write_usize(&mut self, i: usize) { self.0 = i as _ }
    fn write_isize(&mut self, i: isize) { self.0 = i as _ }
}

#[derive(Default)]
pub(crate) struct NullHashBuilder;

impl BuildHasher for NullHashBuilder {
    type Hasher = NullHash;

    fn build_hasher(&self) -> Self::Hasher {
        NullHash(0)
    }
}
