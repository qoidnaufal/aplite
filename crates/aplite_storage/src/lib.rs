mod data_store;
mod entity;
mod index_map;
mod iterator;
mod slot;
mod tree;

pub use tree::*;
pub use entity::{Entity, EntityManager};
pub use index_map::IndexMap;
pub use data_store::*;

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
