use crate::entity::Entity;
use super::tree::Tree;

#[derive(Debug)]
pub struct Node {
    pub entity: Entity,
    pub parent: Option<Entity>,
    pub first_child: Option<Entity>,
    pub next_sibling: Option<Entity>,
    pub prev_sibling: Option<Entity>,
}

impl Node {
    pub(crate) fn as_node_ref(&self) -> NodeRef<'_> {
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
pub struct NodeRef<'a> {
    pub entity: &'a Entity,
    pub parent: Option<&'a Entity>,
    pub first_child: Option<&'a Entity>,
    pub next_sibling: Option<&'a Entity>,
    pub prev_sibling: Option<&'a Entity>,
}

impl NodeRef<'_> {
    pub(crate) fn index(&self) -> usize {
        self.entity.index()
    }
}

impl From<NodeRef<'_>> for Node {
    fn from(value: NodeRef<'_>) -> Self {
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

pub struct SubTree {
    id: Entity,
    nodes: Vec<Node>
}

impl SubTree {
    pub fn new(id: Entity) -> Self {
        Self {
            id,
            nodes: Vec::new(),
        }
    }

    pub(crate) fn from_tree(id: Entity, tree: &Tree) -> Self {
        let nodes = tree.iter_node(&id)
            .skip(1)
            .map(|node_ref| node_ref.into())
            .collect();
        Self {
            id,
            nodes,
        }
    }

    pub fn id(&self) -> &Entity {
        &self.id
    }

    pub fn add_child(&mut self, child: Entity) {
        let child_node = Node {
            entity: child,
            parent: Some(self.id),
            first_child: None,
            next_sibling: None,
            prev_sibling: self.nodes.last().map(|node| node.entity),
        };
        if let Some(last) = self.nodes.last_mut() {
            last.next_sibling = Some(child);
        }
        self.nodes.push(child_node);
    }

    pub fn add_child_node(&mut self, node: Node) {
        if let Some(last) = self.nodes.last_mut() {
            last.next_sibling = Some(node.entity);
        }
        self.nodes.push(node);
    }

    pub fn iter_member_ref(&self) -> impl Iterator<Item = NodeRef<'_>> {
        self.nodes
            .iter()
            .map(|node| node.as_node_ref())
    }

    pub fn len(&self) -> usize {
        self.nodes.len() + 1
    }
}

impl std::fmt::Debug for SubTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(&self.nodes)
            .finish()
    }
}
