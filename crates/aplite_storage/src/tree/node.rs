use crate::entity::Entity;

pub struct Node<E: Entity> {
    pub entity: E,
    pub parent: Option<E>,
    pub first_child: Option<E>,
    pub next_sibling: Option<E>,
    pub prev_sibling: Option<E>,
}

// impl<E: Entity> Node<E> {
//     pub(crate) fn new(entity: E, parent: Option<E>) -> Self {
//         Self {
//             entity,
//             parent,
//             first_child: None,
//             next_sibling: None,
//             prev_sibling: None,
//         }
//     }
// }

pub struct NodeRef<'a, E: Entity> {
    pub entity: &'a E,
    pub parent: Option<&'a E>,
    pub first_child: Option<&'a E>,
    pub next_sibling: Option<&'a E>,
    pub prev_sibling: Option<&'a E>,
}

// pub struct NodeMut<'a, E: Entity> {
//     pub entity: &'a E,
//     pub parent: Option<&'a mut E>,
//     pub first_child: Option<&'a mut E>,
//     pub next_sibling: Option<&'a mut E>,
//     pub prev_sibling: Option<&'a mut E>,
// }
