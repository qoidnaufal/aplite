use std::iter::FilterMap;
use std::slice::{Iter, IterMut};
use std::iter::Enumerate;

use crate::entity::Entity;
use crate::tree::Tree;
use crate::slot::{Slot, Content};
use crate::index_map::IndexMap;
use crate::hash::U64Map;

/*
#########################################################
#                                                       #
#                        NodeRef                        #
#                                                       #
#########################################################
*/

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

/*
#########################################################
#                                                       #
#                        NodeMut                        #
#                                                       #
#########################################################
*/

pub struct NodeMut<'a, E: Entity, T> {
    pub(crate) id: E,
    pub(crate) parent: Option<&'a mut E>,
    pub(crate) first_child: Option<&'a mut E>,
    pub(crate) next_sibling: Option<&'a mut E>,
    pub(crate) data: Option<&'a mut T>,
}

impl<'a, E: Entity, T> NodeMut<'a, E, T> {
    pub(crate) fn new(tree: &'a mut Tree<E, T>, entity: E) -> Self {
        Self {
            id: entity,
            parent: tree.parent[entity.index()].as_mut(),
            first_child: tree.first_child[entity.index()].as_mut(),
            next_sibling: tree.next_sibling[entity.index()].as_mut(),
            data: tree.data.inner[entity.index()].get_content_mut(),
        }
    }

    pub fn id(&self) -> E { self.id }

    pub fn parent(&'a mut self) -> Option<&'a mut E> { self.parent.as_deref_mut() }

    pub fn first_child(&'a mut self) -> Option<&'a mut E> { self.first_child.as_deref_mut() }

    pub fn next_sibling(&'a mut self) -> Option<&'a mut E> { self.next_sibling.as_deref_mut() }

    pub fn data(&'a mut self) -> Option<&'a mut T> { self.data.as_deref_mut() }
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
    type IntoIter = TreeIter<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIter {
            inner: self.data.iter(),
            tree: self,
        }
    }
}

pub struct TreeIter<'a, E: Entity, T> {
    inner: IndexMapIter<'a, E, T>,
    tree: &'a Tree<E, T>
}

impl<'a, E: Entity, T> Iterator for TreeIter<'a, E, T> {
    type Item = NodeRef<'a, E, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(entity, data)| {
                NodeRef::new(self.tree, entity, data)
            })
    }
}

impl<'a, E: Entity, T> DoubleEndedIterator for TreeIter<'a, E, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
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

fn filter_map<'a, E, T>((i, slot): (usize, &'a Slot<T>)) -> Option<(E, Option<&'a T>)>
where
    E: Entity
{
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            E::new(i as u32, slot.version),
            slot.get_content()
        ))
}

impl<'a, E, T> IntoIterator for &'a IndexMap<E, T>
where
    E: Entity,
{
    type Item = (E, &'a T);
    type IntoIter = IndexMapIter<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter()
            .enumerate()
            .filter_map(filter_map as fn((usize, &Slot<T>)) -> Option<(E, Option<&T>)>);

        IndexMapIter {
            inner,
        }
    }
}

pub struct IndexMapIter<'a, E: Entity, T> {
    inner: FilterMap<
        Enumerate<Iter<'a, Slot<T>>>,
        fn((usize, &'a Slot<T>)) -> Option<(E, Option<&'a T>)>
    >,
}

impl<'a, E, T> Iterator for IndexMapIter<'a, E, T>
where
    E: Entity,
{
    type Item = (E, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(e, val)| (e, val.unwrap()))
    }
}

impl<'a, E, T> DoubleEndedIterator for IndexMapIter<'a, E, T>
where
    E: Entity
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(e, val)| (e, val.unwrap()))
    }
}

/*
#########################################################
#                                                       #
#                    &mut INDEX MAP                     #
#                                                       #
#########################################################
*/

fn filter_mut<'a, E, T>((i, slot): (usize, &'a mut Slot<T>)) -> Option<(E, Option<&'a mut T>)>
where
    E: Entity
{
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            E::new(i as u32, slot.version),
            slot.get_content_mut()
        ))
}

impl<'a, E: Entity, T> IntoIterator for &'a mut IndexMap<E, T> {
    type Item = (E, &'a mut T);
    type IntoIter = IndexMapIterMut<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter_mut()
            .enumerate()
            .filter_map(filter_mut as fn((usize, &mut Slot<T>)) -> Option<(E, Option<&mut T>)>);

        IndexMapIterMut { inner }
    }
}

pub struct IndexMapIterMut<'a, E: Entity, T> {
    inner: FilterMap<
        Enumerate<IterMut<'a, Slot<T>>>,
        fn((usize, &'a mut Slot<T>)) -> Option<(E, Option<&'a mut T>)>
    >,
}

impl<'a, E: Entity, T> Iterator for IndexMapIterMut<'a, E, T> {
    type Item = (E, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(e, val)| (e, val.unwrap()))
    }
}

impl<'a, E: Entity, T> DoubleEndedIterator for IndexMapIterMut<'a, E, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(e, val)| (e, val.unwrap()))
    }
}

/*
#########################################################
#                                                       #
#                        U64MAP                         #
#                                                       #
#########################################################
*/

impl<'a, K, V> IntoIterator for &'a U64Map<K, V> {
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
