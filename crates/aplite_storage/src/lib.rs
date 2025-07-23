mod tree;
mod manager;
mod iterator;
mod entity;
mod hash;
mod storage;
mod slot;

pub use tree::*;
pub use manager::EntityManager;
pub use entity::Entity;
pub use hash::Map;
pub use storage::Storage;

#[derive(Debug)]
pub enum Error {
    ReachedMaxId,
    InternalCollision,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
