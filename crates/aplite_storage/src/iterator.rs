use std::iter::FilterMap;
use std::slice::{Iter, IterMut};
use std::iter::{Enumerate, Map, Zip};

use crate::entity::Entity;
use crate::tree::Tree;
use crate::slot::{Slot, Content};
use crate::index_map::IndexMap;
use crate::hash::U64Map;

/*
#########################################################
#                                                       #
#                       INDEX MAP                       #
#                                                       #
#########################################################
*/

fn filter_ref<E, T>((i, slot): (usize, &Slot<T>)) -> Option<(E, Option<&T>)>
where
    E: Entity
{
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            E::new(i as u32, slot.version),
            slot.get_content()
        ))
}

type FnFilterRef<E, T> = fn((usize, &Slot<T>)) -> Option<(E, Option<&T>)>;

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
            .filter_map(filter_ref as FnFilterRef<E, T>);

        IndexMapIter {
            inner,
        }
    }
}

pub struct IndexMapIter<'a, E: Entity, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<Iter<'a, Slot<T>>>, FnFilterRef<E, T>>,
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

fn filter_mut<E, T>((i, slot): (usize, &mut Slot<T>)) -> Option<(E, Option<&mut T>)>
where
    E: Entity
{
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            E::new(i as u32, slot.version),
            slot.get_content_mut()
        ))
}

type FnFilterMut<E, T> = fn((usize, &mut Slot<T>)) -> Option<(E, Option<&mut T>)>;

impl<'a, E: Entity, T> IntoIterator for &'a mut IndexMap<E, T> {
    type Item = (E, &'a mut T);
    type IntoIter = IndexMapIterMut<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter_mut()
            .enumerate()
            .filter_map(filter_mut as FnFilterMut<E, T>);

        IndexMapIterMut { inner }
    }
}

pub struct IndexMapIterMut<'a, E: Entity, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<IterMut<'a, Slot<T>>>, FnFilterMut<E, T>>,
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
    pub(crate) data: &'a mut T,
}

impl<'a, E: Entity, T> NodeMut<'a, E, T> {
    pub(crate) fn new(tree: &'a mut Tree<E, T>, entity: E, data: &'a mut T) -> Self {
        Self {
            id: entity,
            parent: tree.parent[entity.index()].as_mut(),
            first_child: tree.first_child[entity.index()].as_mut(),
            next_sibling: tree.next_sibling[entity.index()].as_mut(),
            data,
        }
    }

    pub fn id(&self) -> E { self.id }

    pub fn parent(&'a mut self) -> Option<&'a mut E> { self.parent.as_deref_mut() }

    pub fn first_child(&'a mut self) -> Option<&'a mut E> { self.first_child.as_deref_mut() }

    pub fn next_sibling(&'a mut self) -> Option<&'a mut E> { self.next_sibling.as_deref_mut() }

    pub fn data(&'a mut self) -> &'a mut T { self.data }
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

fn tree_iter_mut<'a, E, T>(
    ((((id, data), parent), first_child), next_sibling):
        ((((E, &'a mut T), &'a mut Option<E>), &'a mut Option<E>), &'a mut Option<E>)
    ) -> NodeMut<'a, E, T>
where
    E: Entity
{
    NodeMut {
        id,
        parent: parent.as_mut(),
        first_child: first_child.as_mut(),
        next_sibling: next_sibling.as_mut(),
        data,
    }
}

type FnTreeMapIntoNodeMut<'a, E, T> =
    fn(
        ((((E, &'a mut T), &'a mut Option<E>), &'a mut Option<E>), &'a mut Option<E>)
    ) -> NodeMut<'a, E, T>;

impl<'a, E: Entity, T> IntoIterator for &'a mut Tree<E, T> {
    type Item = NodeMut<'a, E, T>;
    type IntoIter = TreeIterMut<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIterMut {
            inner: self
                .data
                .iter_mut()
                .zip(&mut self.parent)
                .zip(&mut self.first_child)
                .zip(&mut self.next_sibling)
                .map(tree_iter_mut as FnTreeMapIntoNodeMut<E, T>)
        }
    }
}

pub struct TreeIterMut<'a, E: Entity, T> {
    inner: Map<Zip<Zip<Zip<IndexMapIterMut<'a, E, T>,
            IterMut<'a, Option<E>>>,
            IterMut<'a, Option<E>>>,
            IterMut<'a, Option<E>>>,
            FnTreeMapIntoNodeMut<'a, E, T>>,
}

impl<'a, E: Entity + 'a, T: 'a> Iterator for TreeIterMut<'a, E, T> {
    type Item = NodeMut<'a, E, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/*
#########################################################
#                                                       #
#                  TREE::child_iterator                 #
#                                                       #
#########################################################
*/

pub struct ChildIterator<'a, E: Entity, T> {
    tree: &'a Tree<E, T>,
    current: Option<&'a E>,
}

impl<'a, E: Entity, T> ChildIterator<'a, E, T> {
    pub(crate) fn new(tree: &'a Tree<E, T>, entity: &'a E) -> Self {
        let current = tree.get_first_child(entity);
        Self {
            tree,
            current,
        }
    }
}

impl<'a, E: Entity, T> Iterator for ChildIterator<'a, E, T> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        if let Some(e) = current {
            self.current = self.tree.get_next_sibling(e)
        }
        current
    }
}

/*
#########################################################
#                                                       #
#                 TREE::member_iterator                 #
#                                                       #
#########################################################
*/

pub struct MemberIterator<'a, E: Entity, T> {
    tree: &'a Tree<E, T>,
    head: Option<&'a E>,
    next: Option<&'a E>,
}

impl<'a, E: Entity, T> MemberIterator<'a, E, T> {
    pub(crate) fn new(tree: &'a Tree<E, T>, entity: &'a E) -> Self {
        let head = tree.get_first_child(entity);
        Self {
            tree,
            head,
            next: None,
        }
    }
}

impl<'a, E: Entity, T> Iterator for MemberIterator<'a, E, T> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        self.next = self.head.take();

        if let Some(prev_head) = self.next {
            self.head = self.tree.get_next_sibling(prev_head);
        }

        let next = self.next.take();
        if let Some(next) = next {
            self.next = self.tree.get_first_child(next);
        }
        next
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
    fn tree_iter() {
        let mut tree = Tree::<TestId, ()>::new();
        let mut ids = vec![];
        for _ in 0..10 {
            let id = tree.insert(());
            ids.push(id);
        }

        let len = tree.iter_node_ref().count();
        assert_eq!(ids.len(), len)
    }

    #[test]
    fn tree_iter_mut() {
        let mut tree = Tree::<TestId, ()>::new();
        let mut ids = vec![];
        for _ in 0..10 {
            let id = tree.insert(());
            ids.push(id);
        }

        let len = tree.iter_node_mut().count();
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
