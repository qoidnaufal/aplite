use std::slice::{Iter, IterMut};
use std::iter::{Enumerate, FilterMap};

use crate::entity::EntityId;
use crate::tree::tree::Tree;
use crate::tree::node::NodeRef;
use crate::indexmap::slot::{Slot, Content};
use crate::indexmap::IndexMap;

/*
#########################################################
#                                                       #
#                       INDEX MAP                       #
#                                                       #
#########################################################
*/

fn index_map_filter_ref<T>((i, slot): (usize, &Slot<T>)) -> Option<(EntityId, Option<&T>)> {
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            EntityId::new(i as u32, slot.version),
            slot.get_content()
        ))
}

type FnIndexMapFilterRef<T> = fn((usize, &Slot<T>)) -> Option<(EntityId, Option<&T>)>;

impl<'a, T> IntoIterator for &'a IndexMap<T> {
    type Item = (EntityId, &'a T);
    type IntoIter = IndexMapIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter()
            .enumerate()
            .filter_map(index_map_filter_ref as FnIndexMapFilterRef<T>);

        IndexMapIter {
            inner,
        }
    }
}

pub struct IndexMapIter<'a, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<Iter<'a, Slot<T>>>, FnIndexMapFilterRef<T>>,
}

impl<'a, T> Iterator for IndexMapIter<'a, T> {
    type Item = (EntityId, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

impl<'a, T> DoubleEndedIterator for IndexMapIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

/*
#########################################################
#                                                       #
#                    &mut INDEX MAP                     #
#                                                       #
#########################################################
*/

fn index_map_filter_mut<T>((i, slot): (usize, &mut Slot<T>)) -> Option<(EntityId, Option<&mut T>)> {
    matches!(slot.content, Content::Occupied(_))
        .then_some((
            EntityId::new(i as u32, slot.version),
            slot.get_content_mut()
        ))
}

type FnIndexMapFilterMut<T> = fn((usize, &mut Slot<T>)) -> Option<(EntityId, Option<&mut T>)>;

impl<'a, T> IntoIterator for &'a mut IndexMap<T> {
    type Item = (EntityId, &'a mut T);
    type IntoIter = IndexMapIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .iter_mut()
            .enumerate()
            .filter_map(index_map_filter_mut as FnIndexMapFilterMut<T>);

        IndexMapIterMut { inner }
    }
}

pub struct IndexMapIterMut<'a, T> {
    #[allow(clippy::type_complexity)]
    inner: FilterMap<Enumerate<IterMut<'a, Slot<T>>>, FnIndexMapFilterMut<T>>,
}

impl<'a, T> Iterator for IndexMapIterMut<'a, T> {
    type Item = (EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

impl<'a, T> DoubleEndedIterator for IndexMapIterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(id, val)| (id, val.unwrap()))
    }
}

/*
#########################################################
#                                                       #
#                    NodeRef Iterator                   #
#                                                       #
#########################################################
*/

pub struct TreeNodeIter<'a> {
    inner: TreeDepthIter<'a>
}

impl<'a> TreeNodeIter<'a> {
    pub(crate) fn new(tree: &'a Tree, id: &'a EntityId) -> Self {
        Self {
            inner: TreeDepthIter::new(tree, id)
        }
    }
}

impl<'a> Iterator for TreeNodeIter<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|entity| {
                NodeRef {
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

pub struct TreeChildIter<'a> {
    tree: &'a Tree,
    next: Option<&'a EntityId>,
    back: Option<&'a EntityId>,
}

impl<'a> TreeChildIter<'a> {
    pub(crate) fn new(tree: &'a Tree, id: &'a EntityId) -> Self {
        let next = tree.get_first_child(id);
        let back = tree.get_last_child(id);
        Self {
            tree,
            next,
            back,
        }
    }
}

impl<'a> Iterator for TreeChildIter<'a> {
    type Item = &'a EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        if let Some(current) = next {
            self.next = self.tree.get_next_sibling(current)
        }
        next
    }
}

impl<'a> DoubleEndedIterator for TreeChildIter<'a> {
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
pub struct TreeDepthIter<'a> {
    tree: &'a Tree,
    id: &'a EntityId,
    next: Option<&'a EntityId>,
}

impl<'a> TreeDepthIter<'a> {
    pub(crate) fn new(tree: &'a Tree, id: &'a EntityId) -> Self {
        Self {
            tree,
            id,
            next: Some(id),
        }
    }
}

impl<'a> Iterator for TreeDepthIter<'a> {
    type Item = &'a EntityId;

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
                    if parent == self.id { break }

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

/*
#########################################################
#                                                       #
#                TREE::ancestor_iterator                #
#                                                       #
#########################################################
*/

pub struct TreeAncestryIter<'a> {
    tree: &'a Tree,
    id: &'a EntityId,
}

impl<'a> TreeAncestryIter<'a> {
    pub(crate) fn new(tree: &'a Tree, id: &'a EntityId) -> Self {
        Self {
            tree,
            id,
        }
    }
}

impl<'a> Iterator for TreeAncestryIter<'a> {
    type Item = &'a EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.tree.get_parent(self.id);
        if let Some(next) = next {
            self.id = next;
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
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod iterator_test {
    use crate::tree::tree::Tree;
    use crate::entity::{EntityId, IdManager};
    use crate::indexmap::IndexMap;

    #[test]
    fn indexmap() {
        let mut storage = IndexMap::<usize>::with_capacity(10);
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

    #[test]
    fn tree_iter() {
        let mut manager = IdManager::default();
        let mut tree = Tree::default();

        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create();
            tree.insert(id, None);
            ids.push(id);
        }

        let len = tree.iter_node(&EntityId::root()).count();
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
