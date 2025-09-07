use std::slice::{Iter, IterMut};
use std::iter::{Enumerate, FilterMap, Zip, Filter};

use crate::entity::Entity;
use crate::tree::{Tree, Node};
use crate::indexmap::slot::{Slot, Content};
use crate::indexmap::IndexMap;
use crate::data_store::DataStore;

/*
#########################################################
#                                                       #
#                       INDEX MAP                       #
#                                                       #
#########################################################
*/

fn index_map_filter_ref<E, T>((i, slot): (usize, &Slot<T>)) -> Option<(E, Option<&T>)>
where
    E: Entity
{
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            E::new(i as u32, slot.version),
            slot.get_content()
        ))
}

type FnIndexMapFilterRef<E, T> = fn((usize, &Slot<T>)) -> Option<(E, Option<&T>)>;

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
            .filter_map(index_map_filter_ref as FnIndexMapFilterRef<E, T>);

        IndexMapIter {
            inner,
        }
    }
}

pub struct IndexMapIter<'a, E: Entity, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<Iter<'a, Slot<T>>>, FnIndexMapFilterRef<E, T>>,
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

fn index_map_filter_mut<E, T>((i, slot): (usize, &mut Slot<T>)) -> Option<(E, Option<&mut T>)>
where
    E: Entity
{
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            E::new(i as u32, slot.version),
            slot.get_content_mut()
        ))
}

type FnIndexMapFilterMut<E, T> = fn((usize, &mut Slot<T>)) -> Option<(E, Option<&mut T>)>;

impl<'a, E: Entity, T> IntoIterator for &'a mut IndexMap<E, T> {
    type Item = (E, &'a mut T);
    type IntoIter = IndexMapIterMut<'a, E, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter_mut()
            .enumerate()
            .filter_map(index_map_filter_mut as FnIndexMapFilterMut<E, T>);

        IndexMapIterMut { inner }
    }
}

pub struct IndexMapIterMut<'a, E: Entity, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<IterMut<'a, Slot<T>>>, FnIndexMapFilterMut<E, T>>,
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
#                    NodeRef Iterator                   #
#                                                       #
#########################################################
*/

pub struct TreeNodeIter<'a, E: Entity> {
    inner: TreeDepthIter<'a, E>
}

impl<'a, E: Entity> TreeNodeIter<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            inner: TreeDepthIter::new(tree, entity)
        }
    }
}

impl<E: Entity> Iterator for TreeNodeIter<'_, E> {
    type Item = Node<E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|entity| {
                Node {
                    entity,
                    parent: self.inner.tree.get_parent(entity),
                    first_child: self.inner.tree.get_first_child(entity),
                    next_sibling: self.inner.tree.get_next_sibling(entity),
                    prev_sibling: self.inner.tree.get_prev_sibling(entity),
                }
            })
    }
}

/*
#########################################################
#                                                       #
#                  TREE::child_iterator                 #
#                                                       #
#########################################################
*/

pub struct TreeChildIter<'a, E: Entity> {
    tree: &'a Tree<E>,
    next: Option<E>,
    back: Option<E>,
}

impl<'a, E: Entity> TreeChildIter<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        let next = tree.get_first_child(entity);
        let back = tree.get_last_child(entity);
        Self {
            tree,
            next,
            back,
        }
    }
}

impl<'a, E: Entity> Iterator for TreeChildIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        if let Some(current) = next {
            self.next = self.tree.get_next_sibling(current)
        }
        next
    }
}

impl<'a, E: Entity> DoubleEndedIterator for TreeChildIter<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let back = self.back.take();
        if let Some(current) = back {
            self.back = self.tree.get_prev_sibling(current)
        }
        back
    }
}

/*
#########################################################
#                                                       #
#                 TREE::member_iterator                 #
#                                                       #
#########################################################
*/

// TODO: Make a double-ended iterator
/// Depth first traversal
pub struct TreeDepthIter<'a, E: Entity> {
    tree: &'a Tree<E>,
    entity: E,
    next: Option<E>,
}

impl<'a, E: Entity> TreeDepthIter<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            tree,
            entity,
            next: Some(entity),
        }
    }
}

impl<'a, E: Entity> Iterator for TreeDepthIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();

        if let Some(current) = next {
            if let Some(first_child) = self.tree.get_first_child(current) {
                self.next = Some(first_child);
            } else if let Some(next_sibling) = self.tree.get_next_sibling(current) {
                self.next = Some(next_sibling);
            } else {
                // arrived at the last child position
                let mut curr = current;

                while let Some(parent) = self.tree.get_parent(curr) {
                    if parent == self.entity { break }

                    if let Some(next_sibling) = self.tree.get_next_sibling(parent) {
                        self.next = Some(next_sibling);
                        break
                    }

                    curr = parent;
                }
            }
        }

        next
    }
}

/// Breadth first traversal
pub struct TreeBreadthIter<'a, E: Entity> {
    tree: &'a Tree<E>,
    queue: std::collections::VecDeque<E>,
    next: Option<E>,
    back: Vec<E>,
}

impl<'a, E: Entity> TreeBreadthIter<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            tree,
            queue: Default::default(),
            next: Some(entity),
            back: Default::default(),
        }
    }
}

impl<'a, E: Entity> Iterator for TreeBreadthIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();

        if let Some(current) = next {
            self.back.push(current);

            if self.tree.get_first_child(current).is_some() {
                self.queue.push_back(current);
            }

            if let Some(next_sibling) = self.tree.get_next_sibling(current) {
                self.next = Some(next_sibling);
            } else if let Some(head) = self.queue.pop_front() {
                self.next = self.tree.get_first_child(head);
            } else {
                self.next = self.tree.get_first_child(current);
            }
        } else {
            self.queue.clear();
        }

        next
    }
}

impl<'a, E: Entity> DoubleEndedIterator for TreeBreadthIter<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next.is_some() {
            for _ in self.by_ref() {}
        }

        self.back.pop()
    }
}

/*
#########################################################
#                                                       #
#                TREE::ancestor_iterator                #
#                                                       #
#########################################################
*/

pub struct TreeAncestryIter<'a, E: Entity> {
    tree: &'a Tree<E>,
    entity: E,
}

impl<'a, E: Entity> TreeAncestryIter<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            tree,
            entity,
        }
    }
}

impl<'a, E: Entity> Iterator for TreeAncestryIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.tree.get_parent(self.entity);
        if let Some(next) = next {
            self.entity = next;
        }
        next
    }
}

/*
#########################################################
#                                                       #
#              TREE::double_ended_iterator              #
#                                                       #
#########################################################
*/

// pub(crate) enum Direction {
//     EnteringFirstChild,
//     EnteringNextSibling,
// }

// pub(crate) struct DirectionalTreeIter<'a, E: Entity> {
//     tree: &'a Tree<E>,
//     next: Option<E>,
//     direction: Direction,
// }

// impl<'a, E: Entity> Iterator for DirectionalTreeIter<'a, E> {
//     type Item = E;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.next.take();
//         if let Some(current) = next {}
//         next
//     }
// }

// impl<'a, E: Entity> DoubleEndedIterator for DirectionalTreeIter<'a, E> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         let next = self.next.take();
//         if let Some(current) = next {
//             if let Some(prev_sibling) = self.tree.get_prev_sibling(current) {}
//         }
//         next
//     }
// }

/*
#########################################################
#                                                       #
#                       DataStore                       #
#                                                       #
#########################################################
*/

fn filter_data_store(idx: &&usize) -> bool {
    idx != &&usize::MAX
}

pub struct DataStoreIter<'a, T> {
    inner: Zip<Filter<Iter<'a, usize>, fn(&&usize) -> bool>,
            Iter<'a, T>>,
}

impl<'a, T> DataStoreIter<'a, T> {
    pub(crate) fn new<E: Entity>(ds: &'a DataStore<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .filter(filter_data_store as fn(&&usize) -> bool)
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for DataStoreIter<'a, T> {
    type Item = (&'a usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn filter_map_data_store<E: Entity>((i, idx): (usize, &usize)) -> Option<E> {
    (idx != &usize::MAX).then_some(E::new(i as u32, 0))
}

pub struct MappedDataStoreIter<'a, E: Entity, T> {
    inner: Zip<FilterMap<Enumerate<Iter<'a, usize>>, fn((usize, &usize)) -> Option<E>>,
            Iter<'a, T>>,
}

impl<'a, E: Entity, T> MappedDataStoreIter<'a, E, T> {
    pub(crate) fn new(ds: &'a DataStore<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .enumerate()
            .filter_map(filter_map_data_store as fn((usize, &usize)) -> Option<E>)
            .zip(ds.data.iter());
        Self {
            inner,
        }
    }
}

impl<'a, E: Entity, T> Iterator for MappedDataStoreIter<'a, E, T> {
    type Item = (E, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn filter_data_store_mut(idx: &&usize) -> bool {
    idx != &&usize::MAX
}

pub struct DataStoreIterMut<'a, T> {
    inner: Zip<Filter<Iter<'a, usize>, fn(&&usize) -> bool>,
            IterMut<'a, T>>,
}

impl<'a, T> DataStoreIterMut<'a, T> {
    pub(crate) fn new<E: Entity>(ds: &'a mut DataStore<E, T>) -> Self {
        let inner = ds.ptr.ptr
            .iter()
            .filter(filter_data_store_mut as fn(&&usize) -> bool)
            .zip(ds.data.iter_mut());
        Self {
            inner,
        }
    }
}

impl<'a, T> Iterator for DataStoreIterMut<'a, T> {
    type Item = (&'a usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/*
#########################################################
#                                                       #
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod iterator_test {
    use crate::tree::Tree;
    use crate::entity::{Entity, EntityManager};
    use crate::indexmap::IndexMap;
    use crate::create_entity;

    create_entity! { TestId }

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
            storage.remove(created_ids[i * 3]);
        }

        let remaining = storage.iter().count();
        assert_eq!(remaining, storage.len());
    }

    #[test]
    fn tree_iter() {
        let mut manager = EntityManager::<TestId>::default();
        let mut tree = Tree::<TestId>::default();

        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create();
            tree.insert_as_parent(id);
            ids.push(id);
        }

        let len = tree.iter_node(TestId::root()).count();
        assert_eq!(ids.len(), len)
    }

    // #[test]
    // fn tree_iter_mut() {
    //     let mut manager = EntityManager::<TestId>::default();
    //     let mut tree = Tree::<TestId>::default();
    //     let root = manager.create();
    //     tree.add_root(root);
    //     let mut ids = vec![];
    //     for _ in 0..10 {
    //         let id = manager.create();
    //         tree.add_child(root, id);
    //         ids.push(id);
    //     }

    //     let len = tree.iter_node_mut().count();
    //     assert_eq!(ids.len(), len)
    // }
}
