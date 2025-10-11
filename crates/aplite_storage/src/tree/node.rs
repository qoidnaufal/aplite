use crate::entity::Entity;
use super::tree::Tree;

#[derive(Debug)]
pub struct Node<E: Entity> {
    pub entity: E,
    pub parent: Option<E>,
    pub first_child: Option<E>,
    pub next_sibling: Option<E>,
    pub prev_sibling: Option<E>,
}

impl<E: Entity> Node<E> {
    pub(crate) fn as_node_ref(&self) -> NodeRef<'_, E> {
        NodeRef {
            entity: &self.entity,
            parent: self.parent.as_ref(),
            first_child: self.first_child.as_ref(),
            next_sibling: self.next_sibling.as_ref(),
            prev_sibling: self.prev_sibling.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct NodeRef<'a, E: Entity> {
    pub entity: &'a E,
    pub parent: Option<&'a E>,
    pub first_child: Option<&'a E>,
    pub next_sibling: Option<&'a E>,
    pub prev_sibling: Option<&'a E>,
}

impl<E: Entity> NodeRef<'_, E> {
    pub(crate) fn index(&self) -> usize {
        self.entity.index()
    }
}

impl<E: Entity> From<NodeRef<'_, E>> for Node<E> {
    fn from(value: NodeRef<'_, E>) -> Self {
        Self {
            entity: *value.entity,
            parent: value.parent.copied(),
            first_child: value.first_child.copied(),
            next_sibling: value.next_sibling.copied(),
            prev_sibling: value.prev_sibling.copied(),
        }
    }
}

// pub struct NodeMut<'a, E: Entity> {
//     pub entity: &'a E,
//     pub parent: Option<&'a mut E>,
//     pub first_child: Option<&'a mut E>,
//     pub next_sibling: Option<&'a mut E>,
//     pub prev_sibling: Option<&'a mut E>,
// }

pub struct SubTree<E: Entity> {
    entity: E,
    nodes: Vec<Node<E>>
}

impl<E: Entity> SubTree<E> {
    pub fn new(entity: E) -> Self {
        Self {
            entity,
            nodes: Vec::new(),
        }
    }

    pub(crate) fn from_tree(entity: E, tree: &Tree<E>) -> Self {
        let nodes = tree.iter_node(&entity)
            .skip(1)
            .map(|node_ref| node_ref.into())
            .collect();
        Self {
            entity,
            nodes,
        }
    }

    pub fn id(&self) -> &E {
        &self.entity
    }

    pub fn add_child(&mut self, child: E) {
        let child_node = Node {
            entity: child,
            parent: Some(self.entity),
            first_child: None,
            next_sibling: None,
            prev_sibling: self.nodes.last().map(|node| node.entity),
        };
        if let Some(last) = self.nodes.last_mut() {
            last.next_sibling = Some(child);
        }
        self.nodes.push(child_node);
    }

    pub fn add_child_node(&mut self, node: Node<E>) {
        if let Some(last) = self.nodes.last_mut() {
            last.next_sibling = Some(node.entity);
        }
        self.nodes.push(node);
    }

    pub fn iter_member_ref(&self) -> impl Iterator<Item = NodeRef<'_, E>> {
        self.nodes
            .iter()
            .map(|node| node.as_node_ref())
    }

    pub fn len(&self) -> usize {
        self.nodes.len() + 1
    }
}

impl<E: Entity> std::fmt::Debug for SubTree<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(&self.nodes)
            .finish()
    }
}
