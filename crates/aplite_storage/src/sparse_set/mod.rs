pub(crate) mod typed;
pub(crate) mod type_erased;
pub(crate) mod indices;

use crate::entity::EntityId;
use crate::data::component::ComponentId;
use crate::map::slot_map::SlotId;

pub trait SparsetKey
where
    Self: Clone + Copy + PartialEq + Eq + std::fmt::Debug + 'static
{
    fn index(&self) -> usize;
}

impl SparsetKey for crate::entity::Entity {
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

impl SparsetKey for SlotId {
    fn index(&self) -> usize {
        self.index()
    }
}
