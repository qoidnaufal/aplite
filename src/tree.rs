use std::sync::atomic::{AtomicU64, Ordering};

const INITIAL_CAPACITY: usize = 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub(crate) u64);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl NodeId {
    pub(crate) const fn root() -> Self { Self(0) }

    pub(crate) fn new() -> Self {
        static NODE_ID: AtomicU64 = AtomicU64::new(1);
        Self(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Entity for NodeId {
    fn new() -> Self { Self::new() }
    
    fn root() -> Self { Self::root() }

    fn index(&self) -> usize { self.0 as usize }
}

pub trait Entity: Copy + PartialEq + PartialOrd {
    fn new() -> Self;
    fn root() -> Self;
    fn index(&self) -> usize;
}

#[derive(Debug)]
pub(crate) struct Tree<E: Entity> {
    entities: Vec<E>,
    parent: Vec<Option<E>>,
    first_child: Vec<Option<E>>,
    last_child: Vec<Option<E>>,
    next_sibling: Vec<Option<E>>,
    prev_sibling: Vec<Option<E>>,
}

impl<E: Entity> Default for Tree<E> {
    fn default() -> Self { Self::new() }
}

impl<E: Entity> Tree<E> {
    fn new() -> Self {
        Self {
            entities: Vec::with_capacity(INITIAL_CAPACITY),
            parent: Vec::with_capacity(INITIAL_CAPACITY),
            first_child: Vec::with_capacity(INITIAL_CAPACITY),
            last_child: Vec::with_capacity(INITIAL_CAPACITY),
            next_sibling: Vec::with_capacity(INITIAL_CAPACITY),
            prev_sibling: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }

    pub(crate) fn create_entity(&self) -> E { E::new() }

    pub(crate) fn iter(&self) -> TreeIterator<'_, E> { self.into_iter() }

    pub(crate) fn insert(&mut self, entity: E, maybe_parent: Option<E>) {
        self.entities.push(entity);
        self.first_child.push(None);
        self.last_child.push(None);
        self.next_sibling.push(None);
        self.prev_sibling.push(None);
        self.parent.push(maybe_parent);
        if let Some(parent) = maybe_parent.as_ref() {
            self.add_child(parent, entity);
        }
    }

    #[allow(unused)]
    pub(crate) fn get_ancestor<'a>(&'a self, entity: &'a E) -> Option<&'a E> {
        if let Some(parent) = self.get_parent(entity) {
            self.get_ancestor(parent)
        } else { Some(entity) }
    }

    pub(crate) fn get_parent(&self, entity: &E) -> Option<&E> {
        self.parent.get(entity.index()).and_then(|e| e.as_ref())
    }

    pub(crate) fn get_first_child(&self, entity: &E) -> Option<&E> {
        self.first_child.get(entity.index()).and_then(|e| e.as_ref())
    }

    pub(crate) fn get_last_child(&self, entity: &E) -> Option<&E> {
        self.last_child.get(entity.index()).and_then(|e| e.as_ref())
    }

    #[allow(unused)]
    pub(crate) fn get_next_sibling(&self, entity: &E) -> Option<&E> {
        self.next_sibling.get(entity.index()).and_then(|e| e.as_ref())
    }

    #[allow(unused)]
    pub(crate) fn get_prev_sibling(&self, entity: &E) -> Option<&E> {
        self.prev_sibling.get(entity.index()).and_then(|e| e.as_ref())
    }

    #[allow(unused)]
    pub(crate) fn get_parent_mut(&mut self, entity: &E) -> Option<&mut E> {
        self.parent.get_mut(entity.index()).and_then(|e| e.as_mut())
    }

    #[allow(unused)]
    pub(crate) fn get_first_child_mut(&mut self, entity: &E) -> Option<&mut E> {
        self.first_child.get_mut(entity.index()).and_then(|e| e.as_mut())
    }

    #[allow(unused)]
    pub(crate) fn get_last_child_mut(&mut self, entity: &E) -> Option<&mut E> {
        self.last_child.get_mut(entity.index()).and_then(|e| e.as_mut())
    }

    pub(crate) fn get_next_sibling_mut(&mut self, entity: &E) -> Option<&mut E> {
        self.next_sibling.get_mut(entity.index()).and_then(|e| e.as_mut())
    }

    pub(crate) fn get_prev_sibling_mut(&mut self, entity: &E) -> Option<&mut E> {
        self.prev_sibling.get_mut(entity.index()).and_then(|e| e.as_mut())
    }

    pub(crate) fn get_all_children(&self, entity: &E) -> Option<Vec<&E>> {
        if let Some(first) = self.get_first_child(entity) {
            let last = self.get_last_child(entity).unwrap();
            if first == last {
                return Some(vec![first]);
            } else {
                assert!(last > first);
                self.entities[first.index()..=last.index()]
                    .iter()
                    .map(|child_entity| Some(child_entity))
                    .collect()
            }
        } else {
            None
        }
    }

    pub(crate) fn add_child(&mut self, entity: &E, child: E) {
        match self.get_first_child(entity).cloned() {
            Some(first) => {
                let last = *self.get_last_child(entity).unwrap();
                if last == first {
                    self.set_prev_sibling(&child, first);
                    self.set_next_sibling(&first, child);
                } else {
                    assert!(last > first);
                    self.set_prev_sibling(&child, last);
                    self.set_next_sibling(&last, child);
                }
            },
            None => self.set_first_child(entity, child),
        }
        self.set_last_child(entity, child);
    }

    pub(crate) fn set_first_child(&mut self, entity: &E, child: E) {
        if self.first_child.len() > entity.index() {
            self.first_child[entity.index()] = Some(child);
        } else {
            self.first_child.push(Some(child));
        }
    }

    pub(crate) fn set_last_child(&mut self, entity: &E, child: E) {
        if self.last_child.len() > entity.index() {
            self.last_child[entity.index()] = Some(child);
        } else {
            self.last_child.push(Some(child));
        }
    }

    pub(crate) fn set_prev_sibling(&mut self, entity: &E, prev: E) {
        if let Some(prev_sibling) = self.get_prev_sibling_mut(entity) {
            *prev_sibling = prev;
        } else {
            if self.prev_sibling.len() > entity.index() {
                self.prev_sibling[entity.index()] = Some(prev);
            } else {
                self.prev_sibling.push(Some(prev));
            }
        }
    }

    pub(crate) fn set_next_sibling(&mut self, entity: &E, next: E) {
        if let Some(next_sibling) = self.get_next_sibling_mut(entity) {
            *next_sibling = next;
        } else {
            if self.next_sibling.len() > entity.index() {
                self.next_sibling[entity.index()] = Some(next);
            } else {
                self.next_sibling.push(Some(next));
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    #[allow(unused)]
    pub(crate) fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub(crate) fn get_node_ref(&self, index: usize) -> NodeRef<'_, E> {
        NodeRef::new(self, index)
    }
}

pub struct NodeRef<'a, E: Entity> {
    id: &'a E,
    parent: Option<&'a E>,
    first_child: Option<&'a E>,
    last_child: Option<&'a E>,
    next_sibling: Option<&'a E>,
    prev_sibling: Option<&'a E>,
}

impl<'a, E: Entity> NodeRef<'a, E> {
    fn new(tree: &'a Tree<E>, idx: usize) -> Self {
        Self {
            id: &tree.entities[idx],
            parent: tree.parent[idx].as_ref(),
            first_child: tree.first_child[idx].as_ref(),
            last_child: tree.last_child[idx].as_ref(),
            next_sibling: tree.next_sibling[idx].as_ref(),
            prev_sibling: tree.prev_sibling[idx].as_ref(),
        }
    }

    pub(crate) fn id(&self) -> &'a E { self.id }

    #[allow(unused)]
    pub(crate) fn parent(&self) -> Option<&'a E> { self.parent }

    #[allow(unused)]
    pub(crate) fn first_child(&self) -> Option<&'a E> { self.first_child }

    #[allow(unused)]
    pub(crate) fn last_child(&self) -> Option<&'a E> { self.last_child }

    #[allow(unused)]
    pub(crate) fn next_sibling(&self) -> Option<&'a E> { self.next_sibling }

    #[allow(unused)]
    pub(crate) fn prev_sibling(&self) -> Option<&'a E> { self.prev_sibling }
}

pub struct TreeIterator<'a, E: Entity> {
    tree: &'a Tree<E>,
    counter: usize,
}

impl<'a, E: Entity> TreeIterator<'a, E> {
    fn new(tree: &'a Tree<E>) -> Self {
        Self { tree, counter: 0 }
    }
}

impl<'a, E: Entity> Iterator for TreeIterator<'a, E> {
    type Item = NodeRef<'a, E>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.tree.len() {
            let node = Some(self.tree.get_node_ref(self.counter));
            self.counter += 1;
            return node;
        } else {
            None
        }
    }
}

impl<'a, E: Entity> IntoIterator for &'a Tree<E> {
    type Item = NodeRef<'a, E>;
    type IntoIter = TreeIterator<'a, E>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIterator::new(self)
    }
}

#[cfg(test)]
mod tree_test {
    use super::*;

    fn setup_tree() -> Tree<NodeId> {
        let mut tree: Tree<NodeId> = Tree::new();
        tree.insert(NodeId::root(), None);
        let mut parent = None;
        for i in 0..10 {
            let node_id = tree.create_entity();
            tree.insert(node_id, parent);
            if i > 0 && i % 3 == 0 {
                parent = Some(NodeId(1));
            } else {
                parent = Some(node_id);
            }
        }
        tree
    }

    #[test]
    fn get_ancestor() {
        let tree = setup_tree();

        let ancestor = tree.get_ancestor(&NodeId(10));
        let next_sibling = tree.get_next_sibling(&NodeId(5));
        assert_eq!(ancestor, Some(&NodeId(1)));
        assert_eq!(next_sibling, Some(&NodeId(8)));
    }
}
