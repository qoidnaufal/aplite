pub(crate) mod typed;
pub(crate) mod type_erased;
pub(crate) mod indices;

use crate::entity::{EntityId, Entity};
use crate::data::component::ComponentId;

pub trait SparsetKey
where
    Self: Clone + Copy + PartialEq + Eq + std::fmt::Debug + 'static
{
    fn index(&self) -> usize;
}

impl SparsetKey for Entity {
    fn index(&self) -> usize {
        self.index()
    }
}

impl SparsetKey for EntityId {
    fn index(&self) -> usize {
        self.0 as _
    }
}

impl SparsetKey for ComponentId {
    fn index(&self) -> usize {
        self.0 as _
    }
}
