use std::slice::{Iter, IterMut};
use std::iter::{Enumerate, FilterMap};

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
#                        NodeRef                        #
#                                                       #
#########################################################
*/

pub struct NodeRef<E: Entity> {
    pub(crate) entity: E,
    pub(crate) parent: Option<E>,
    pub(crate) first_child: Option<E>,
    pub(crate) next_sibling: Option<E>,
}

impl<E: Entity> NodeRef<E> {
    pub fn id(&self) -> E { self.entity }

    pub fn parent(&self) -> Option<E> { self.parent }

    pub fn first_child(&self) -> Option<E> { self.first_child }

    pub fn next_sibling(&self) -> Option<E> { self.next_sibling }
}

/*
#########################################################
#                                                       #
#                        NodeMut                        #
#                                                       #
#########################################################
*/

// pub struct NodeMut<'a, E: Entity> {
//     pub(crate) entity: E,
//     pub(crate) parent: Option<&'a mut E>,
//     pub(crate) first_child: Option<&'a mut E>,
//     pub(crate) next_sibling: Option<&'a mut E>,
// }

// impl<'a, E: Entity> NodeMut<'a, E> {
//     pub fn id(&'a mut self) -> E { self.entity }

//     pub fn parent(&'a mut self) -> Option<&'a mut E> { self.parent.as_deref_mut() }

//     pub fn first_child(&'a mut self) -> Option<&'a mut E> { self.first_child.as_deref_mut() }

//     pub fn next_sibling(&'a mut self) -> Option<&'a mut E> { self.next_sibling.as_deref_mut() }
// }

/*
#########################################################
#                                                       #
#                         TREE                          #
#                                                       #
#########################################################
*/

pub struct TreeNodeIterator<'a, E: Entity> {
    inner: MemberIterator<'a, E>
}

impl<'a, E: Entity> TreeNodeIterator<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            inner: MemberIterator::new(tree, entity)
        }
    }
}

impl<E: Entity> Iterator for TreeNodeIterator<'_, E> {
    type Item = NodeRef<E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|entity| {
                NodeRef {
                    entity,
                    parent: self.inner.tree.get_parent(entity),
                    first_child: self.inner.tree.get_first_child(entity),
                    next_sibling: self.inner.tree.get_next_sibling(entity),
                }
            })
    }
}

/*
#########################################################
#                                                       #
#                       &mut TREE                       #
#                                                       #
#########################################################
*/

// fn is_some_mut<'a, E>(
//     (((e, p), f), n):
//         (((&mut Option<E>, &'a mut Option<E>), &'a mut Option<E>), &'a mut Option<E>)
//     ) -> Option<NodeMut<'a, E>>
// where
//     E: Entity
// {
//     e.map(|e| NodeMut {
//         entity: e,
//         parent: p.as_mut(),
//         first_child: f.as_mut(),
//         next_sibling: n.as_mut(),
//     })
// }

// type FnIsSomeMut<'a, E> =
//     fn(
//         (((&mut Option<E>, &'a mut Option<E>), &'a mut Option<E>), &'a mut Option<E>)
//     ) -> Option<NodeMut<'a, E>>;

// impl<'a, E: Entity> IntoIterator for &'a mut Tree<E> {
//     type Item = NodeMut<'a, E>;
//     type IntoIter = TreeIterMut<'a, E>;

//     fn into_iter(self) -> Self::IntoIter {
//         TreeIterMut {
//             inner: self
//                 .entities
//                 .iter_mut()
//                 .zip(&mut self.parent)
//                 .zip(&mut self.first_child)
//                 .zip(&mut self.next_sibling)
//                 .filter_map(is_some_mut as FnIsSomeMut<'a, E>)
//         }
//     }
// }

// pub struct TreeIterMut<'a, E: Entity> {
//     inner: FilterMap<Zip<Zip<Zip<
//             IterMut<'a, Option<E>>, IterMut<'a, Option<E>>>,
//             IterMut<'a, Option<E>>>, IterMut<'a, Option<E>>>,
//             FnIsSomeMut<'a, E>>,
// }

// impl<'a, E: Entity + 'a> Iterator for TreeIterMut<'a, E> {
//     type Item = NodeMut<'a, E>;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next()
//     }
// }

/*
#########################################################
#                                                       #
#                  TREE::child_iterator                 #
#                                                       #
#########################################################
*/

pub struct ChildIterator<'a, E: Entity> {
    tree: &'a Tree<E>,
    current: Option<E>,
}

impl<'a, E: Entity> ChildIterator<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        let current = tree.get_first_child(entity);
        Self {
            tree,
            current,
        }
    }
}

impl<'a, E: Entity> Iterator for ChildIterator<'a, E> {
    type Item = E;

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

// pub(crate) enum IterDirection {
//     EnteringFirstChild,
//     EnteringNextSibling,
// }

// TODO: Make a double-ended iterator
/// Depth first Iterator
pub struct MemberIterator<'a, E: Entity> {
    tree: &'a Tree<E>,
    entity: E,
    next: Option<E>,
}

impl<'a, E: Entity> MemberIterator<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            tree,
            entity,
            next: tree.get_first_child(entity),
        }
    }
}

impl<'a, E: Entity> Iterator for MemberIterator<'a, E> {
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
                    if parent == self.entity {
                        break
                    }

                    if let Some(next_sibling) = self.tree.get_next_sibling(parent) {
                        self.next = Some(next_sibling);
                    }

                    curr = parent;
                }
            }
        }

        next
    }
}

/*
#########################################################
#                                                       #
#                TREE::ancestor_iterator                #
#                                                       #
#########################################################
*/

pub struct AncestorIter<'a, E: Entity> {
    tree: &'a Tree<E>,
    entity: E,
}

impl<'a, E: Entity> AncestorIter<'a, E> {
    pub(crate) fn new(tree: &'a Tree<E>, entity: E) -> Self {
        Self {
            tree,
            entity,
        }
    }
}

impl<'a, E: Entity> Iterator for AncestorIter<'a, E> {
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
    use crate::entity::{Entity, EntityManager};
    use crate::index_map::IndexMap;
    use crate::entity;

    entity! { TestId }

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

        let root = manager.create();
        tree.add_root(root);

        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create();
            tree.add_child(root, id);
            ids.push(id);
        }

        let len = tree.iter_node(root).count();
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
