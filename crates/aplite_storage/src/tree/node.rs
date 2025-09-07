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
