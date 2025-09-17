use crate::entity::Entity;
use crate::data::dense_column::DenseColumn;

pub struct Node<E: Entity> {
    pub entity: E,
    pub parent: Option<E>,
    pub first_child: Option<E>,
    pub next_sibling: Option<E>,
    pub prev_sibling: Option<E>,
}

impl<E: Entity> Node<E> {
    pub(crate) fn new(entity: E, parent: Option<E>) -> Self {
        Self {
            entity,
            parent,
            first_child: None,
            next_sibling: None,
            prev_sibling: None,
        }
    }
}

pub struct NodeRef<'a, E: Entity> {
    pub entity: &'a E,
    pub parent: Option<&'a E>,
    pub first_child: Option<&'a E>,
    pub next_sibling: Option<&'a E>,
    pub prev_sibling: Option<&'a E>,
}

pub struct NodeMut<'a, E: Entity> {
    pub entity: &'a E,
    pub parent: Option<&'a mut E>,
    pub first_child: Option<&'a mut E>,
    pub next_sibling: Option<&'a mut E>,
    pub prev_sibling: Option<&'a mut E>,
}

pub struct SubTree<E: Entity> {
    pub(crate) inner: DenseColumn<E, Node<E>>,
}

impl<E: Entity> SubTree<E> {
    pub fn get_node_ref<'a>(&'a self, entity: &'a E) -> Option<NodeRef<'a, E>> {
        self.inner
            .get(entity)
            .map(|node| {
                NodeRef {
                    entity,
                    parent: node.parent.as_ref(),
                    first_child: node.first_child.as_ref(),
                    next_sibling: node.next_sibling.as_ref(),
                    prev_sibling: node.prev_sibling.as_ref(),
                }
            })
    }

    pub fn get_node_mut<'a>(&'a mut self, entity: &'a E) -> Option<NodeMut<'a, E>> {
        self.inner
            .get_mut(entity)
            .map(|node| {
                NodeMut {
                    entity,
                    parent: node.parent.as_mut(),
                    first_child: node.first_child.as_mut(),
                    next_sibling: node.next_sibling.as_mut(),
                    prev_sibling: node.prev_sibling.as_mut(),
                }
            })
    }

    pub fn get_parent(&self, entity: &E) -> Option<&E> {
        self.inner.get(entity).and_then(|node| node.parent.as_ref())
    }

    pub fn get_next_sibling(&self, entity: &E) -> Option<&E> {
        self.inner.get(entity).and_then(|node| node.next_sibling.as_ref())
    }

    pub fn get_prev_sibling(&self, entity: &E) -> Option<&E> {
        self.inner.get(entity).and_then(|node| node.prev_sibling.as_ref())
    }

    pub fn get_first_child(&self, entity: &E) -> Option<&E> {
        self.inner.get(entity).and_then(|node| node.first_child.as_ref())
    }

    pub fn insert(&mut self, entity: E, parent: Option<E>) {
        let node = Node::new(entity, parent);
        self.inner.insert(&entity, node);

        if let Some(parent) = parent && let Some(parent_node) = self.inner.get(&parent) {
            if let Some(first_child) = parent_node.first_child {
                let mut current = first_child;
                while let Some(next) = self.get_next_sibling(&current) {
                    current = *next;
                }
                self.set_next_sibling(&current, entity);
                self.set_prev_sibling(&entity, current);
            } else {
                self.set_first_child(&parent, entity);
            }
        }
    }

    pub fn set_next_sibling(&mut self, entity: &E, sibling: E) {
        if let Some(node) = self.inner.get_mut(entity) {
            node.next_sibling = Some(sibling);
        }
    }

    pub fn set_prev_sibling(&mut self, entity: &E, prev_sibling: E) {
        if let Some(node) = self.inner.get_mut(entity) {
            node.prev_sibling = Some(prev_sibling);
        }
    }

    pub fn set_first_child(&mut self, entity: &E, child: E) {
        if let Some(node) = self.inner.get_mut(entity) {
            node.first_child = Some(child);
        }
    }

    pub fn remove(&mut self, entity: E) {
        if let Some(node) = self.inner.get_mut(&entity) {}
    }
}
