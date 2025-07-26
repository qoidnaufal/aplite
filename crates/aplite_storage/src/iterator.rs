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

impl<'a, E: Entity> IntoIterator for &'a Tree<E> {
    type Item = NodeRef<'a, E>;
    type IntoIter = TreeIterator<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIterator::new(self)
    }
}

pub struct TreeIterator<'a, E: Entity> {
    tree: &'a Tree<E>,
    counter: usize,
}

pub struct NodeRef<'a, E: Entity> {
    id: &'a E,
    parent: Option<&'a E>,
    first_child: Option<&'a E>,
    next_sibling: Option<&'a E>,
}

impl<'a, E: Entity> NodeRef<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: &'a E) -> Self {
        Self {
            id: entity,
            parent: tree.parent[entity.index()].as_ref(),
            first_child: tree.first_child[entity.index()].as_ref(),
            next_sibling: tree.next_sibling[entity.index()].as_ref(),
        }
    }

    pub fn id(&self) -> &'a E { self.id }

    pub fn parent(&self) -> Option<&'a E> { self.parent }

    pub fn first_child(&self) -> Option<&'a E> { self.first_child }

    pub fn next_sibling(&self) -> Option<&'a E> { self.next_sibling }
}

impl<'a, E: Entity> TreeIterator<'a, E> {
    fn new(tree: &'a Tree<E>) -> Self {
        Self { tree, counter: 0 }
    }
}

impl<'a, E: Entity> Iterator for TreeIterator<'a, E> {
    type Item = NodeRef<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tree
            .manager
            .stored
            .iter()
            .filter(|slot| matches!(slot.content, Content::Occupied(_)))
            .nth(self.counter)
            .map(|slot| {
                self.counter += 1;
                let entity = slot.get_content().unwrap();
                self.tree.get_node_ref(entity)
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.counter, Some(self.tree.len()))
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
        let mut tree = Tree::<TestId>::new();
        let mut ids = vec![];
        for _ in 0..10 {
            let id = tree.create_entity();
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
