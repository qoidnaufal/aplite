use crate::entity::Entity;
use crate::tree::Tree;
use crate::slot::Content;
use crate::index_map::IndexMap;
use crate::hash::Map;

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
    inner: IndexMapIterator<'a, E, T>,
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
#                       INDEX MAP                       #
#                                                       #
#########################################################
*/

impl<'a, E, T: 'a> IntoIterator for &'a IndexMap<E, T>
where
    E: Entity,
{
    type Item = (E, &'a T);
    type IntoIter = IndexMapIterator<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        IndexMapIterator {
            inner: self,
            counter: 0,
        }
    }
}

pub struct IndexMapIterator<'a, E: Entity, T: 'a> {
    inner: &'a IndexMap<E, T>,
    counter: usize,
}

impl<'a, E, T> Iterator for IndexMapIterator<'a, E, T>
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
    use crate::entity::Entity;
    use crate::index_map::IndexMap;
    use crate::entity;

    entity! { TestId }

    #[test]
    fn tree() {
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
    fn indexmap() {
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
