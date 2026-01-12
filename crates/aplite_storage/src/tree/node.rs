use crate::map::id::SlotId;
use super::sparse_tree::SparseTree;

#[derive(Debug)]
pub struct Node {
    pub entity: SlotId,
    pub parent: Option<SlotId>,
    pub first_child: Option<SlotId>,
    pub next_sibling: Option<SlotId>,
    pub prev_sibling: Option<SlotId>,
}

impl Node {
    // pub(crate) fn id(&self) -> EntityId {
    //     self.entity
    // }

    pub(crate) fn index(&self) -> usize {
        self.entity.index()
    }

    // pub(crate) fn as_node_ref(&self) -> NodeRef<'_> {
    //     NodeRef {
    //         entity: self.entity,
    //         parent: self.parent.as_ref(),
    //         first_child: self.first_child.as_ref(),
    //         next_sibling: self.next_sibling.as_ref(),
    //         prev_sibling: self.prev_sibling.as_ref(),
    //     }
    // }
}

#[derive(Debug)]
pub struct NodeRef<'a> {
    pub entity: SlotId,
    pub parent: Option<&'a SlotId>,
    pub first_child: Option<&'a SlotId>,
    pub next_sibling: Option<&'a SlotId>,
    pub prev_sibling: Option<&'a SlotId>,
}

impl NodeRef<'_> {
    // pub(crate) fn index(&self) -> usize {
    //     self.entity.index()
    // }
}

// pub struct NodeMut<'a, E: Entity> {
//     pub entity: &'a E,
//     pub parent: Option<&'a mut E>,
//     pub first_child: Option<&'a mut E>,
//     pub next_sibling: Option<&'a mut E>,
//     pub prev_sibling: Option<&'a mut E>,
// }

pub struct SubTree {
    id: SlotId,
    nodes: Vec<Node>
}

impl SubTree {
    pub fn new(id: SlotId) -> Self {
        Self {
            id,
            nodes: Vec::new(),
        }
    }

    pub(crate) fn from_tree(id: SlotId, tree: &SparseTree) -> Self {
        let nodes = tree.iter_node(id)
            .skip(1)
            .collect();
        Self {
            id,
            nodes,
        }
    }

    pub fn id(&self) -> &SlotId {
        &self.id
    }

    pub fn add_child(&mut self, child: SlotId) {
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

    pub fn iter_member_ref(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
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
