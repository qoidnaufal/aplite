use crate::manager::EntityManager;
use crate::entity::Entity;
use crate::tree::Tree;
use crate::slot::Content;
use crate::index_map::IndexMap;
use crate::hash::Map;

/*
#########################################################
#                                                       #
#                     ENTITY MANAGER                    #
#                                                       #
#########################################################
*/

impl<'a, E: Entity> IntoIterator for &'a EntityManager<E> {
    type Item = &'a E;
    type IntoIter = EntityIterator<'a, E>;
    fn into_iter(self) -> Self::IntoIter {
        EntityIterator {
            inner: self,
            counter: 0,
        }
    }
}

// WARN: this extra allocation is kinda unpleasant, find a way to work around later
pub struct EntityIterator<'a, E: Entity> {
    inner: &'a EntityManager<E>,
    counter: usize,
}

impl<'a, E: Entity> Iterator for EntityIterator<'a, E> {
    type Item = &'a E;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .stored
            .iter()
            .filter(|slot| matches!(slot.content, Content::Occupied(_)))
            .nth(self.counter)
            .map(|slot| {
                self.counter += 1;
                slot.get_content().unwrap()
            })
    }
}

/*
#########################################################
#                                                       #
#                         TREE                          #
#                                                       #
#########################################################
*/

impl<'a, E: Entity, T> IntoIterator for &'a Tree<E, T> {
    type Item = NodeRef<'a, E, T>;
    type IntoIter = TreeIterator<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIterator::new(self)
    }
}

pub struct TreeIterator<'a, E: Entity, T> {
    inner: StorageIterator<'a, E, T>,
    tree: &'a Tree<E, T>
}

pub struct NodeRef<'a, E: Entity, T> {
    id: E,
    parent: Option<&'a E>,
    first_child: Option<&'a E>,
    next_sibling: Option<&'a E>,
    data: &'a T,
}

impl<'a, E: Entity, T> NodeRef<'a, E, T> {
    pub(crate) fn new(tree: &'a Tree<E, T>, entity: E, data: &'a T) -> Self {
        Self {
            id: entity,
            parent: tree.parent[entity.index()].as_ref(),
            first_child: tree.first_child[entity.index()].as_ref(),
            next_sibling: tree.next_sibling[entity.index()].as_ref(),
            data,
        }
    }

    pub fn id(&self) -> E { self.id }

    pub fn parent(&self) -> Option<&'a E> { self.parent }

    pub fn first_child(&self) -> Option<&'a E> { self.first_child }

    pub fn next_sibling(&self) -> Option<&'a E> { self.next_sibling }

    pub fn data(&self) -> &'a T { self.data }
}

impl<'a, E: Entity, T> TreeIterator<'a, E, T> {
    fn new(tree: &'a Tree<E, T>) -> Self {
        Self { inner: tree.data.iter(), tree }
    }
}

impl<'a, E: Entity, T> Iterator for TreeIterator<'a, E, T> {
    type Item = NodeRef<'a, E, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(entity, data)| {
                NodeRef::new(self.tree, entity, data)
            })
    }
}

/*
#########################################################
#                                                       #
#                        STORAGE                        #
#                                                       #
#########################################################
*/

impl<'a, E, T: 'a> IntoIterator for &'a IndexMap<E, T>
where
    E: Entity,
{
    type Item = (E, &'a T);
    type IntoIter = StorageIterator<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        StorageIterator {
            inner: self,
            counter: 0,
        }
    }
}

pub struct StorageIterator<'a, E: Entity, T: 'a> {
    inner: &'a IndexMap<E, T>,
    counter: usize,
}

impl<'a, E, T> Iterator for StorageIterator<'a, E, T>
where
    E: Entity,
{
    type Item = (E, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .inner
            .iter()
            .enumerate()
            .filter(|(_, slot)| matches!(slot.content, Content::Occupied(_)))
            .nth(self.counter)
            .map(|(i, slot)| {
                self.counter += 1;
                (E::new(i as u64, slot.version), slot.get_content().unwrap())
            })
    }
}

/*
#########################################################
#                                                       #
#                          MAP                          #
#                                                       #
#########################################################
*/

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod iterator_test {
    use crate::tree::Tree;
    use crate::manager::EntityManager;
    use crate::entity::Entity;
    use crate::index_map::IndexMap;
    use crate::entity;

    entity! { TestId }

    #[test]
    fn tree_iterator() {
        let mut tree = Tree::<TestId, ()>::new();
        let mut ids = vec![];
        for _ in 0..10 {
            let id = tree.insert(());
            ids.push(id);
        }

        let len = tree.iter().count();
        assert_eq!(ids.len(), len)
    }

    #[test]
    fn entity_iterator() {
        let mut manager = EntityManager::<TestId>::new();
        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create_entity();
            ids.push(id);
        }

        let len = manager.iter().count();
        assert_eq!(ids.len(), len);
    }

    #[test]
    fn storage_iterator() {
        let mut storage = IndexMap::<TestId, usize>::with_capacity(10);
        let mut created_ids = vec![];

        for i in 0..10 {
            let id = storage.insert(i);
            created_ids.push(id);
        }

        assert_eq!(storage.len(), created_ids.len());

        for i in 0..3 {
            storage.remove(&created_ids[i * 3]);
        }

        let remaining = storage.iter().count();
        assert_eq!(remaining, storage.len());
    }
}
