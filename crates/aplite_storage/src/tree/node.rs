use crate::entity::Entity;

pub struct Node<E: Entity> {
    pub(crate) entity: E,
    pub(crate) parent: Option<E>,
    pub(crate) first_child: Option<E>,
    pub(crate) next_sibling: Option<E>,
    pub(crate) prev_sibling: Option<E>,
}

impl<E: Entity> Node<E> {
    pub fn entity(&self) -> E { self.entity }

    pub fn parent(&self) -> Option<E> { self.parent }

    pub fn first_child(&self) -> Option<E> { self.first_child }

    pub fn next_sibling(&self) -> Option<E> { self.next_sibling }

    pub fn prev_sibling(&self) -> Option<E> { self.prev_sibling }
}

pub struct NodeRef<'a, E: Entity> {
    pub(crate) entity: &'a E,
    pub(crate) parent: Option<&'a E>,
    pub(crate) first_child: Option<&'a E>,
    pub(crate) next_sibling: Option<&'a E>,
    pub(crate) prev_sibling: Option<&'a E>,
}

impl<'a, E: Entity> NodeRef<'a, E> {
    pub fn entity(&self) -> &'a E { self.entity }

    pub fn parent(&self) -> Option<&'a E> { self.parent }

    pub fn first_child(&self) -> Option<&'a E> { self.first_child }

    pub fn next_sibling(&self) -> Option<&'a E> { self.next_sibling }

    pub fn prev_sibling(&self) -> Option<&'a E> { self.prev_sibling }
}

pub struct SubTree<E: Entity> {
    pub(crate) inner: Vec<Node<E>>
}
