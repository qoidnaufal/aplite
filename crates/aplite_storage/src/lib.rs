mod tree;
mod iterator;
mod entity;
mod hash;
mod index_map;
mod slot;

pub use tree::*;
pub use entity::{Entity, EntityManager};
pub use hash::U64Map;
pub use index_map::IndexMap;

#[derive(Debug)]
pub enum Error {
    ReachedMaxId,
    InternalCollision,
    InvalidId,
    InvalidSlot,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
