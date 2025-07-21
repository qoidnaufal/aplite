use crate::entity_manager::{Entity, EntityManager};
use crate::tree::Tree;

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
            inner: self.get_entities(),
            counter: 0,
        }
    }
}

// WARN: this extra allocation is kinda unpleasant, find a way to work around later
pub struct EntityIterator<'a, E: Entity> {
    inner: Vec<&'a E>,
    counter: usize,
}

impl<'a, E: Entity> Iterator for EntityIterator<'a, E> {
    type Item = &'a E;
    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.inner.len() {
            let entity = self.inner[self.counter];
            self.counter += 1;
            Some(entity)
        } else {
            None
        }
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
    entities: Vec<&'a E>,
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
        let entities = tree.get_all_entities();
        Self { entities, tree, counter: 0 }
    }
}

impl<'a, E: Entity> Iterator for TreeIterator<'a, E> {
    type Item = NodeRef<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.tree.len() {
            let node = Some(self.tree.get_node_ref(self.entities[self.counter]));
            self.counter += 1;
            node
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.counter, Some(self.tree.len()))
    }
}

#[cfg(test)]
mod iterator_test {
    use crate::tree::Tree;
    use crate::EntityManager;
    use crate::entity_manager::Entity;
    use crate::entity;

    #[test]
    fn tree_iterator() {
        entity! { NodeId }

        let mut tree = Tree::<NodeId>::new();
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
        entity! { NodeId }

        let mut manager = EntityManager::<NodeId>::new();
        let mut ids = vec![];
        for _ in 0..10 {
            let id = manager.create_entity();
            ids.push(id);
        }

        let len = manager.iter().count();
        assert_eq!(ids.len(), len);
    }
}
