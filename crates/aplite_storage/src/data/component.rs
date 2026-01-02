use crate::data::component_storage::ComponentStorage;
use crate::entity::EntityId;

pub trait Component {
    type Item;

    fn insert(self, entity: EntityId, storage: &mut ComponentStorage);
}

pub trait ComponentEq: Component {
    fn component_eq(&self, other: &Self) -> bool;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentId(pub(crate) usize);

impl ComponentId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.0
    }
}

impl std::fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentId({})", self.0)
    }
}

impl std::hash::Hash for ComponentId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/*
#########################################################
#
# TEST
#
#########################################################
*/

#[cfg(test)]
mod component_test {
    use super::*;
    use crate::entity::*;

    crate::make_component!(struct Age(u8));
    crate::make_component!(struct Name(String));
    crate::make_component!(struct Salary(usize));
    crate::make_component!(struct Cars(usize));
    // crate::make_component!(struct Kids((Name, Age)));
    // crate::make_component!(struct Person { name: Name, age: Age });

    #[test]
    fn register_bundle() {
        let mut storage = ComponentStorage::new();
        let mut manager = EntityManager::new();

        let balo = manager.create().id();
        storage.insert_component(balo, (Age(69), Name("Balo".to_string())));
        storage.insert_component(balo, (Salary(6969), Cars(666)));

        let nunez = manager.create().id();
        storage.insert_component(nunez, (Age(69), Name("Balo".to_string())));
    }
}
