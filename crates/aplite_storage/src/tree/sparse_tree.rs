use crate::map::id::SlotId;
use super::node::SubTree;
use crate::tree::node::Node;

/// Sparse array based data structure, where the related information is allocated parallel to the main [`EntityId`].
/// This should enable fast and efficient indexing when accessing the data.
/// This Tree can contains more than one roots.
pub struct SparseTree {
    pub(crate) parent: Vec<Option<SlotId>>,
    pub(crate) first_child: Vec<Option<SlotId>>,
    pub(crate) next_sibling: Vec<Option<SlotId>>,
    pub(crate) prev_sibling: Vec<Option<SlotId>>,
}

impl Default for SparseTree {
    /// This will create a default [`Tree`] without preallocating an initial capacity.
    /// If you want to specify the initial capacity, use [`Tree::with_capacity()`]
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl SparseTree {
    /// Create a new [`Tree`] with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut this = Self {
            parent: Vec::with_capacity(capacity + 1),
            first_child: Vec::with_capacity(capacity + 1),
            next_sibling: Vec::with_capacity(capacity + 1),
            prev_sibling: Vec::with_capacity(capacity + 1),
        };

        this.parent.push(None);
        this.first_child.push(None);
        this.next_sibling.push(None);
        this.prev_sibling.push(None);

        this
    }

    pub fn roots(&self) -> impl Iterator<Item = SlotId> {
        self.parent
            .iter()
            .enumerate()
            .filter_map(|(i, parent)| {
                parent.is_none()
                    .then_some(SlotId::new(i as u32, 0))
            })
    }

    /// get the root of an entity
    pub fn get_root(&self, id: SlotId) -> Option<SlotId> {
        let mut current = id;
        while let Some(parent) = self.get_parent(current) {
            current = parent;
        }
        if current == id {
            None
        } else {
            Some(current)
        }
    }

    #[inline(always)]
    pub fn get_parent(&self, id: SlotId) -> Option<SlotId> {
        self.parent[id.index()]
    }

    #[inline(always)]
    pub fn get_first_child(&self, id: SlotId) -> Option<SlotId> {
        self.first_child[id.index()]
    }

    #[inline(always)]
    pub fn get_last_child(&self, id: SlotId) -> Option<SlotId> {
        let Some(first) = self.get_first_child(id) else { return None };
        let mut last = first;
        while let Some(next) = self.get_next_sibling(last) {
            last = next;
        }
        Some(last)
    }

    #[inline(always)]
    pub fn get_next_sibling(&self, id: SlotId) -> Option<SlotId> {
        self.next_sibling[id.index()]
    }

    #[inline(always)]
    pub fn get_prev_sibling(&self, id: SlotId) -> Option<SlotId> {
        self.prev_sibling[id.index()]
    }

    #[inline(always)]
    pub fn child_count(&self, id: SlotId) -> usize {
        self.iter_children(id).count()
    }

    /// This method will create an allocation,
    /// If you want to avoid unnecessary allocation use [`iter_children`](Self::iter_children)
    #[inline(always)]
    pub fn get_all_children(&self, id: SlotId) -> Vec<SlotId> {
        self.iter_children(id).collect()
    }

    #[inline(always)]
    fn resize_if_needed(&mut self, index: usize) {
        if index >= self.parent.len() {
            self.parent.resize(index + 1, None);
            self.first_child.resize(index + 1, None);
            self.next_sibling.resize(index + 1, None);
            self.prev_sibling.resize(index + 1, None);
        }
    }

    pub fn insert(&mut self, id: SlotId, parent: Option<SlotId>) {
        if let Some(parent) = parent {
            self.insert_with_parent(id, parent);
        } else {
            self.insert_as_root(id);
        }
    }

    #[inline(always)]
    pub fn insert_as_root(&mut self, id: SlotId) {
        self.resize_if_needed(id.index());
    }

    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent,
    /// or the next sibling of parent's last child.
    #[inline(always)]
    pub fn insert_with_parent(&mut self, id: SlotId, parent: SlotId) {
        self.try_insert_with_parent(id, parent).unwrap()
    }

    #[inline(always)]
    /// Adding an entity to be the child of a `maybe parent`.
    /// This will calculate if it's the first child of the parent, or the next sibling of parent's last child.
    /// If a [`TreeError`] is returned it means that the parent is invalid, usually because you haven't registered it to the tree
    pub fn try_insert_with_parent(&mut self, id: SlotId, parent: SlotId) -> Result<(), TreeError> {
        let parent_index = parent.index();
        if parent_index >= self.parent.len() { return Err(TreeError::InvalidEntityId) }

        let index = id.index();
        self.resize_if_needed(index);
        self.parent[index] = Some(parent);

        if let Some(last) = self.get_last_child(parent) {
            self.next_sibling[last.index()] = Some(id);
            self.prev_sibling[index] = Some(last);
        } else {
            self.first_child[parent_index] = Some(id);
        }

        Ok(())
    }

    /// Add a sibling to an entity. This will check the current sibling of the entity.
    /// If [`None`], immediately sets the next sibling. If [`Some`], loop until find the last sibling.
    pub fn add_sibling(&mut self, id: SlotId, sibling: SlotId) {
        self.try_add_sibling(id, sibling).unwrap()
    }

    /// Add a sibling to an entity. This will check if the provided entity is a valid one or not.
    /// If the returned result is [`TreeError`], this means the provided entity is either not registered,
    /// or is actually a root. Maybe you want to add a root instead
    #[inline(always)]
    pub fn try_add_sibling(&mut self, id: SlotId, sibling: SlotId) -> Result<(), TreeError> {
        if id.index() >= self.parent.len() { return Err(TreeError::InvalidEntityId) }

        let Some(parent) = self.get_parent(id) else { return Err(TreeError::InvalidEntityId) };

        let parent = parent;
        self.insert_with_parent(sibling, parent);

        Ok(())
    }

    #[inline(always)]
    pub fn insert_subtree(&mut self, subtree: SubTree, parent: Option<SlotId>) {
        self.insert(*subtree.id(), parent);
        subtree.iter_member_ref()
            .for_each(|node_ref| {
                self.insert(node_ref.entity, node_ref.parent);
            });
    }

    #[inline(always)]
    pub fn detach_if_needed(&mut self, id: SlotId) {
        if self.contains(id) {
            self.detach(id);
        }
    }

    #[inline(always)]
    pub fn detach(&mut self, id: SlotId) {
        let prev = self.get_prev_sibling(id);
        let next = self.get_next_sibling(id);

        if let Some(prev) = prev {
            self.next_sibling[prev.index()] = next;
        } else if let Some(parent) = self.get_parent(id) {
            self.first_child[parent.index()] = next;
        }

        if let Some(next) = next {
            self.prev_sibling[next.index()] = prev;
        }
    }

    #[inline(always)]
    pub fn set_child(&mut self, id: SlotId, child: SlotId) {
        self.detach(child);
        self.insert_with_parent(child, id);
    }

    #[inline(always)]
    /// Currently produces another Tree with the member of the removed entity.
    /// Kinda inefficient if the entity has super big index.
    pub fn remove(&mut self, id: SlotId) -> Self {
        let mut removed_branch = Self::default();
        removed_branch.insert_as_root(id);

        self.iter_node(id)
            .for_each(|node| {
                removed_branch.insert(node.entity, node.parent);
            });

        self.detach(id);

        removed_branch
            .iter_depth(id)
            .for_each(|removed| {
                let index = removed.index();

                self.parent[index] = None;
                self.first_child[index] = None;
                self.next_sibling[index] = None;
                self.prev_sibling[index] = None;
            });

        removed_branch
    }

    pub fn remove_subtree(&mut self, id: SlotId) -> SubTree {
        let subtree = SubTree::from_tree(id, self);

        self.detach(id);

        subtree
            .iter_member_ref()
            .for_each(|node| {
                let index = node.index();

                self.parent[index] = None;
                self.first_child[index] = None;
                self.next_sibling[index] = None;
                self.prev_sibling[index] = None;
            });

        subtree
    }

    /// the distance of an entity from the root
    pub fn entity_depth(&self, id: SlotId) -> usize {
        let mut current = id;
        let mut depth = 0;
        while let Some(parent) = self.get_parent(current) {
            depth += 1;
            current = parent;
        }
        depth
    }

    pub fn tree_depth(&self) -> usize {
        1 + self.first_child
            .iter()
            .filter(|p| p.is_some())
            .count()
    }

    fn ancestors_with_sibling(&self, id: SlotId) -> Vec<bool> {
        let mut current = id;
        let mut loc = vec![];
        while let Some(parent) = self.get_parent(current) {
            loc.push(self.get_next_sibling(parent).is_some());
            current = parent;
        }
        loc.reverse();
        loc
    }

    pub fn is_member_of(&self, id: SlotId, ancestor: SlotId) -> bool {
        if self.get_first_child(ancestor).is_none() {
            return false
        }
        let mut check = id;
        while let Some(parent) = self.get_parent(check) {
            if parent == ancestor {
                return true;
            }
            check = parent
        }
        check == ancestor
    }

    pub fn len(&self, start: SlotId) -> usize {
        self.iter_depth(start).count()
    }

    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    pub fn contains(&self, id: SlotId) -> bool {
        id.index() <= self.parent.len()
        && (
            self.parent.contains(&Some(id))
            || self.first_child.contains(&Some(id))
            || self.next_sibling.contains(&Some(id))
        )
    }

    pub fn reset(&mut self) {
        self.parent.clear();
        self.first_child.clear();
        self.next_sibling.clear();
        self.prev_sibling.clear();

        self.parent.push(None);
        self.first_child.push(None);
        self.next_sibling.push(None);
        self.prev_sibling.push(None);
    }

    /// iterate the children of the entity
    pub fn iter_children<'a>(&'a self, id: SlotId) -> TreeChildIter<'a> {
        TreeChildIter::new(self, id)
    }

    /// iterate the members of the entity
    pub fn iter_depth<'a>(&'a self, id: SlotId) -> TreeDepthIter<'a> {
        TreeDepthIter::new(self, id)
    }

    /// iterate the entity's parent upward
    pub fn iter_ancestry<'a>(&'a self, id: SlotId) -> TreeAncestryIter<'a> {
        TreeAncestryIter::new(self, id)
    }

    /// iterate the member of the iterator and map it to a [`NodeRef`](crate::iterator::NodeRef)
    pub fn iter_node<'a>(&'a self, id: SlotId) -> TreeNodeIter<'a> {
        TreeNodeIter::new(self, id)
    }

    #[inline(always)]
    fn get_frame<'a>(&self, id: SlotId) -> &'a str {
        match self.get_next_sibling(id) {
            Some(_) => "├─",
            None => "└─",
        }
    }

    pub fn recursively_fill_string_buffer(&self, start: Option<SlotId>, s: &mut String) {
        match start {
            Some(parent) => {
                self.iter_children(parent).for_each(|child| {
                    let ancestor_sibling = self.ancestors_with_sibling(child);
                    let loc = ancestor_sibling
                        .iter()
                        .enumerate()
                        .map(|(i, val)| val.then_some(i).unwrap_or_default())
                        .max()
                        .unwrap_or_default();

                    let depth = self.entity_depth(child);
                    let frame = self.get_frame(child);
                    let len = frame.len() / 2;

                    let mut connector_indent = 0;
                    for yes in ancestor_sibling {
                        let mut reducer = 0;
                        if yes {
                            s.push_str(format!("{:connector_indent$}│", "").as_str());
                            connector_indent = 0;
                            reducer = 1;
                        }
                        connector_indent += len - reducer;
                    }

                    let modifier = if loc > 0 { 1 } else { 0 };
                    let indent = len * (depth - loc) - modifier;
                    let format = format!("{:indent$}{frame} {child:?}\n", "");
                    s.push_str(format.as_str());

                    self.recursively_fill_string_buffer(Some(child), s);
                });
            },
            None => {
                self.roots().for_each(|root| {
                    s.push_str(format!(" > {root:?}\n").as_str());
                    self.recursively_fill_string_buffer(Some(root), s);
                });
            },
        }
    }
}

/*
#########################################################
#                                                       #
#                         PRINT                         #
#                                                       #
#########################################################
*/

// FIXME: there are two spot which created unnecessary allocation on get_all_children + get_all_roots
impl std::fmt::Debug for SparseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        self.recursively_fill_string_buffer(None, &mut s);
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub enum TreeError {
    InvalidParent,
    InvalidEntityId,
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TreeError {}

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
    pub(crate) fn new(tree: &'a SparseTree, id: SlotId) -> Self {
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
    next: Option<SlotId>,
    back: Option<SlotId>,
}

impl<'a> TreeChildIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: SlotId) -> Self {
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
    type Item = SlotId;

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
    id: SlotId,
    next: Option<SlotId>,
}

impl<'a> TreeDepthIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: SlotId) -> Self {
        Self {
            tree,
            id,
            next: Some(id),
        }
    }
}

impl<'a> Iterator for TreeDepthIter<'a> {
    type Item = SlotId;

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
    id: SlotId,
}

impl<'a> TreeAncestryIter<'a> {
    pub(crate) fn new(tree: &'a SparseTree, id: SlotId) -> Self {
        Self {
            tree,
            id,
        }
    }
}

impl<'a> Iterator for TreeAncestryIter<'a> {
    type Item = SlotId;

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
mod tree_test {
    use super::*;
    use crate::map::slot_map::SlotMap;

    fn setup_tree(num: usize) -> (SlotMap<()>, SparseTree) {
        let mut manager = SlotMap::new();
        let root = manager.insert(());
        let mut tree = SparseTree::with_capacity(num);
        let mut parent = Some(root);
        for i in 0..num {
            let id = manager.insert(());
            tree.insert(id, parent);
            if i > 0 && i % 3 == 0 {
                parent = tree.get_first_child(SlotId::new(1, 0));
            } else {
                parent = Some(id);
            }
        }

        (manager, tree)
    }

    #[test]
    fn tree_test() {
        let (_, tree) = setup_tree(11);
        // eprintln!("{tree:?}");
        // eprintln!("{:?}", tree.parent);

        let ancestor_id = SlotId::new(9, 0);
        let ancestor = tree.get_root(ancestor_id);

        let test_id_6 = SlotId::new(6, 0);
        let parent = tree.get_parent(test_id_6);
        let four_is_mem_of_two = tree.is_member_of(SlotId::new(4, 0), SlotId::new(2, 0));
        let nine_is_mem_of_two = tree.is_member_of(SlotId::new(9, 0), SlotId::new(2, 0));

        let test_id_4 = SlotId::new(4, 0);
        let next_sibling = tree.get_next_sibling(test_id_4);

        assert_eq!(ancestor, Some(SlotId::new(0, 0)));
        assert_eq!(parent, Some(SlotId::new(5, 0)));
        assert_eq!(four_is_mem_of_two, nine_is_mem_of_two);
        assert_eq!(next_sibling, None);
    }

    #[test]
    fn member_depth_test() {
        let (_, tree) = setup_tree(11);
        // eprintln!("{tree:?}");

        let root = SlotId::new(0, 0);
        let root_children = tree.iter_children(root).count();
        let all = tree.iter_children(root)
            .map(|id| tree.iter_depth(id).count())
            .sum::<usize>();

        assert_eq!(all + root_children, tree.len(root));

        let subtree_len = tree.len(SlotId::new(5, 0));

        assert_eq!(subtree_len, 3);
    }

    #[test]
    fn remove_first_child() {
        let (_, mut tree) = setup_tree(11);
        let root = SlotId::new(0, 0);
        let initial_len = tree.len(root);

        // eprintln!("{tree:?}");

        let test_id2 = SlotId::new(2, 0);
        let first_child = tree.get_first_child(test_id2).unwrap();
        assert_eq!(first_child, SlotId::new(3, 0));

        let removed = tree.remove(SlotId::new(3, 0));
        let removed_len = removed.len(SlotId::new(3, 0));
        let after_remove_len = tree.len(root);
        assert_eq!(removed_len, 2);
        assert_eq!(after_remove_len, initial_len - removed_len);

        // eprintln!("{removed:?}");
        // eprintln!("{tree:?}");

        // let first_child_after_removal = tree.get_first_child(&test_id2);
    }

    #[test]
    fn remove_sub_tree() {
        let (_, mut tree) = setup_tree(11);
        let root = SlotId::new(0, 0);
        let len = tree.len(root);
        // eprintln!("{tree:?}");

        let id = SlotId::new(8, 0);
        let removed = tree.remove_subtree(id);
        let removed_len = removed.len();
        // eprintln!("{removed:?}");

        let after_remove_len = tree.len(root);
        assert_eq!(len, removed_len + after_remove_len);

        tree.insert_subtree(removed, None);
        // eprintln!("{tree:?}");
    }

    #[test]
    fn sibling_test() {
        let (mut manager, mut tree) = setup_tree(11);
        let existing_entity = SlotId::new(6, 0);
        let new_id = manager.insert(());
        let err_add = tree.try_add_sibling(new_id, existing_entity);
        assert!(err_add.is_err());

        let ok_add = tree.try_add_sibling(existing_entity, new_id);
        assert!(ok_add.is_ok());
    }
}
