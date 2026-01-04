use crate::entity::EntityId;
use crate::tree::sparse_tree::SparseTree;
use crate::tree::node::Node;

/*
#########################################################
#                                                       #
#                    TreeNode Iterator                  #
#                                                       #
#########################################################
*/

pub struct TreeNodeIter<'a> {
    inner: TreeDepthIter<'a>
}

impl<'a> TreeNodeIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: EntityId) -> Self {
        Self {
            inner: TreeDepthIter::new(tree, id)
        }
    }
}

impl<'a> Iterator for TreeNodeIter<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|id| {
                Node {
                    entity: id,
                    parent: self.inner.tree.get_parent(id),
                    first_child: self.inner.tree.get_first_child(id),
                    next_sibling: self.inner.tree.get_next_sibling(id),
                    prev_sibling: self.inner.tree.get_prev_sibling(id),
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
    tree: &'a SparseTree,
    next: Option<EntityId>,
    back: Option<EntityId>,
}

impl<'a> TreeChildIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: EntityId) -> Self {
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
    type Item = EntityId;

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
#                 TREE::depth_iterator                  #
#                                                       #
#########################################################
*/

// TODO: Make a double-ended iterator
/// Depth first traversal
pub struct TreeDepthIter<'a> {
    tree: &'a SparseTree,
    id: EntityId,
    next: Option<EntityId>,
}

impl<'a> TreeDepthIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: EntityId) -> Self {
        Self {
            tree,
            id,
            next: Some(id),
        }
    }
}

impl<'a> Iterator for TreeDepthIter<'a> {
    type Item = EntityId;

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
    tree: &'a SparseTree,
    id: EntityId,
}

impl<'a> TreeAncestryIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: EntityId) -> Self {
        Self {
            tree,
            id,
        }
    }
}

impl<'a> Iterator for TreeAncestryIter<'a> {
    type Item = EntityId;

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
    use crate::tree::sparse_tree::SparseTree;
    use crate::entity::{EntityId, EntityManager};

    #[test]
    fn tree_iter() {
        let mut manager = EntityManager::default();
        let mut tree = SparseTree::default();

        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create().id();
            tree.insert_as_root(id);
            ids.push(id);
        }

        let len = tree.iter_node(EntityId::new(0)).count();
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
