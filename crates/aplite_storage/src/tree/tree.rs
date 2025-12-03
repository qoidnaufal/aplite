use crate::entity::EntityId;
use crate::iterator::{
    TreeChildIter,
    TreeDepthIter,
    TreeAncestryIter,
    TreeNodeIter,
};
use super::node::SubTree;

/// Sparse array based data structure, where the related information is allocated parallel to the main [`EntityId`].
/// This should enable fast and efficient indexing when accessing the data.
/// This Tree can contains more than one roots.
pub struct Tree {
    pub(crate) parent: Vec<Option<EntityId>>,
    pub(crate) first_child: Vec<Option<EntityId>>,
    pub(crate) next_sibling: Vec<Option<EntityId>>,
    pub(crate) prev_sibling: Vec<Option<EntityId>>,
}

impl Default for Tree {
    /// This will create a default [`Tree`] without preallocating an initial capacity.
    /// If you want to specify the initial capacity, use [`Tree::with_capacity()`]
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl Tree {
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

    pub fn roots(&self) -> impl Iterator<Item = EntityId> {
        self.parent
            .iter()
            .enumerate()
            .filter_map(|(i, parent)| {
                parent.is_none()
                    .then_some(EntityId::new(i as u32))
            })
    }

    /// get the root of an entity
    pub fn get_root<'a>(&'a self, id: &'a EntityId) -> Option<&'a EntityId> {
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
    pub fn get_parent<'a>(&'a self, id: &'a EntityId) -> Option<&'a EntityId> {
        self.parent[id.index()].as_ref()
    }

    #[inline(always)]
    pub fn get_first_child<'a>(&'a self, id: &'a EntityId) -> Option<&'a EntityId> {
        self.first_child[id.index()].as_ref()
    }

    #[inline(always)]
    pub fn get_last_child<'a>(&'a self, id: &'a EntityId) -> Option<&'a EntityId> {
        let Some(first) = self.get_first_child(id) else { return None };
        let mut last = first;
        while let Some(next) = self.get_next_sibling(last) {
            last = next;
        }
        Some(last)
    }

    #[inline(always)]
    pub fn get_next_sibling<'a>(&'a self, id: &'a EntityId) -> Option<&'a EntityId> {
        self.next_sibling[id.index()].as_ref()
    }

    #[inline(always)]
    pub fn get_prev_sibling<'a>(&'a self, id: &'a EntityId) -> Option<&'a EntityId> {
        self.prev_sibling[id.index()].as_ref()
    }

    #[inline(always)]
    pub fn child_count(&self, id: &EntityId) -> usize {
        self.iter_children(id).count()
    }

    /// This method will create an allocation,
    /// If you want to avoid unnecessary allocation use [`iter_children`](Self::iter_children)
    #[inline(always)]
    pub fn get_all_children<'a>(&'a self, id: &'a EntityId) -> Vec<&'a EntityId> {
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

    pub fn insert(&mut self, id: EntityId, parent: Option<&EntityId>) {
        if let Some(parent) = parent {
            self.insert_with_parent(id, parent);
        } else {
            self.insert_as_root(id);
        }
    }

    #[inline(always)]
    pub fn insert_as_root(&mut self, id: EntityId) {
        self.resize_if_needed(id.index());
    }

    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent,
    /// or the next sibling of parent's last child.
    #[inline(always)]
    pub fn insert_with_parent(&mut self, id: EntityId, parent: &EntityId) {
        self.try_insert_with_parent(id, parent).unwrap()
    }

    #[inline(always)]
    /// Adding an entity to be the child of a `maybe parent`.
    /// This will calculate if it's the first child of the parent, or the next sibling of parent's last child.
    /// If a [`TreeError`] is returned it means that the parent is invalid, usually because you haven't registered it to the tree
    pub fn try_insert_with_parent(&mut self, id: EntityId, parent: &EntityId) -> Result<(), TreeError> {
        let parent_index = parent.index();
        if parent_index >= self.parent.len() { return Err(TreeError::InvalidEntityId) }

        let index = id.index();
        self.resize_if_needed(index);
        self.parent[index] = Some(*parent);

        if let Some(last) = self.get_last_child(parent).copied() {
            self.next_sibling[last.index()] = Some(id);
            self.prev_sibling[index] = Some(last);
        } else {
            self.first_child[parent_index] = Some(id);
        }

        Ok(())
    }

    /// Add a sibling to an entity. This will check the current sibling of the entity.
    /// If [`None`], immediately sets the next sibling. If [`Some`], loop until find the last sibling.
    pub fn add_sibling(&mut self, id: &EntityId, sibling: EntityId) {
        self.try_add_sibling(id, sibling).unwrap()
    }

    /// Add a sibling to an entity. This will check if the provided entity is a valid one or not.
    /// If the returned result is [`TreeError`], this means the provided entity is either not registered,
    /// or is actually a root. Maybe you want to add a root instead
    #[inline(always)]
    pub fn try_add_sibling(&mut self, id: &EntityId, sibling: EntityId) -> Result<(), TreeError> {
        if id.index() >= self.parent.len() { return Err(TreeError::InvalidEntityId) }

        let Some(parent) = self.get_parent(id) else { return Err(TreeError::InvalidEntityId) };

        let parent = *parent;
        self.insert_with_parent(sibling, &parent);

        Ok(())
    }

    #[inline(always)]
    pub fn insert_subtree(&mut self, subtree: SubTree, parent: Option<&EntityId>) {
        self.insert(*subtree.id(), parent);
        subtree.iter_member_ref()
            .for_each(|node_ref| {
                self.insert(*node_ref.entity, node_ref.parent);
            });
    }

    #[inline(always)]
    pub fn detach_if_needed(&mut self, id: &EntityId) {
        if self.contains(id) {
            self.detach(id);
        }
    }

    #[inline(always)]
    pub fn detach(&mut self, id: &EntityId) {
        let prev = self.get_prev_sibling(id).copied();
        let next = self.get_next_sibling(id).copied();

        if let Some(prev) = prev {
            self.next_sibling[prev.index()] = next;
        } else if let Some(parent) = self.get_parent(id).copied() {
            self.first_child[parent.index()] = next;
        }

        if let Some(next) = next {
            self.prev_sibling[next.index()] = prev;
        }
    }

    #[inline(always)]
    pub fn set_child(&mut self, id: &EntityId, child: EntityId) {
        self.detach(&child);
        self.insert_with_parent(child, id);
    }

    #[inline(always)]
    /// Currently produces another Tree with the member of the removed entity.
    /// Kinda inefficient if the entity has super big index.
    pub fn remove(&mut self, id: EntityId) -> Self {
        let mut removed_branch = Self::default();
        removed_branch.insert_as_root(id);

        self.iter_node(&id)
            .for_each(|node| {
                removed_branch.insert(*node.entity, node.parent);
            });

        self.detach(&id);

        removed_branch
            .iter_depth(&id)
            .for_each(|removed| {
                let index = removed.index();

                self.parent[index] = None;
                self.first_child[index] = None;
                self.next_sibling[index] = None;
                self.prev_sibling[index] = None;
            });

        removed_branch
    }

    pub fn remove_subtree(&mut self, id: EntityId) -> SubTree {
        let subtree = SubTree::from_tree(id, self);

        self.detach(&id);

        subtree
            .iter_member_ref()
            .for_each(|node_ref| {
                let index = node_ref.index();

                self.parent[index] = None;
                self.first_child[index] = None;
                self.next_sibling[index] = None;
                self.prev_sibling[index] = None;
            });

        subtree
    }

    /// the distance of an entity from the root
    pub fn entity_depth(&self, id: &EntityId) -> usize {
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

    fn ancestors_with_sibling(&self, id: &EntityId) -> Vec<bool> {
        let mut current = id;
        let mut loc = vec![];
        while let Some(parent) = self.get_parent(current) {
            loc.push(self.get_next_sibling(parent).is_some());
            current = parent;
        }
        loc.reverse();
        loc
    }

    pub fn is_member_of(&self, id: &EntityId, ancestor: &EntityId) -> bool {
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

    pub fn len(&self, start: &EntityId) -> usize {
        self.iter_depth(start).count()
    }

    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        id.index() <= self.parent.len()
        && (
            self.parent.contains(&Some(*id))
            || self.first_child.contains(&Some(*id))
            || self.next_sibling.contains(&Some(*id))
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
    pub fn iter_children<'a>(&'a self, id: &'a EntityId) -> TreeChildIter<'a> {
        TreeChildIter::new(self, id)
    }

    /// iterate the members of the entity
    pub fn iter_depth<'a>(&'a self, id: &'a EntityId) -> TreeDepthIter<'a> {
        TreeDepthIter::new(self, id)
    }

    /// iterate the entity's parent upward
    pub fn iter_ancestry<'a>(&'a self, id: &'a EntityId) -> TreeAncestryIter<'a> {
        TreeAncestryIter::new(self, id)
    }

    /// iterate the member of the iterator and map it to a [`NodeRef`](crate::iterator::NodeRef)
    pub fn iter_node<'a>(&'a self, id: &'a EntityId) -> TreeNodeIter<'a> {
        TreeNodeIter::new(self, id)
    }

    #[inline(always)]
    fn get_frame<'a>(&self, id: &EntityId) -> &'a str {
        match self.get_next_sibling(id) {
            Some(_) => "├─",
            None => "└─",
        }
    }

    pub fn recursively_fill_string_buffer(&self, start: Option<&EntityId>, s: &mut String) {
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
                    self.recursively_fill_string_buffer(Some(&root), s);
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
impl std::fmt::Debug for Tree {
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
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod tree_test {
    use super::*;
    use crate::{EntityId, EntityManager};

    fn setup_tree(num: usize) -> (EntityManager, Tree) {
        let mut manager = EntityManager::default();
        let root = manager.create().id;
        let mut tree = Tree::with_capacity(num);
        let mut parent = Some(root);
        for i in 0..num {
            let id = manager.create().id;
            tree.insert(id, parent.as_ref());
            if i > 0 && i % 3 == 0 {
                parent = tree.get_first_child(&EntityId::new(1)).copied();
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

        let ancestor_id = EntityId::new(9);
        let ancestor = tree.get_root(&ancestor_id);

        let test_id_6 = EntityId::new(6);
        let parent = tree.get_parent(&test_id_6);
        let four_is_mem_of_two = tree.is_member_of(&EntityId::new(4), &EntityId::new(2));
        let nine_is_mem_of_two = tree.is_member_of(&EntityId::new(9), &EntityId::new(2));

        let test_id_4 = EntityId::new(4);
        let next_sibling = tree.get_next_sibling(&test_id_4);

        assert_eq!(ancestor, Some(&EntityId::new(0)));
        assert_eq!(parent, Some(&EntityId::new(5)));
        assert_eq!(four_is_mem_of_two, nine_is_mem_of_two);
        assert_eq!(next_sibling, None);
    }

    #[test]
    fn member_depth_test() {
        let (_, tree) = setup_tree(11);
        // eprintln!("{tree:?}");

        let root = EntityId::new(0);
        let root_children = tree.iter_children(&root).count();
        let all = tree.iter_children(&root)
            .map(|id| tree.iter_depth(id).count())
            .sum::<usize>();

        assert_eq!(all + root_children, tree.len(&root));

        let subtree_len = tree.len(&EntityId::new(5));

        assert_eq!(subtree_len, 3);
    }

    #[test]
    fn remove_first_child() {
        let (_, mut tree) = setup_tree(11);
        let root = EntityId::new(0);
        let initial_len = tree.len(&root);

        // eprintln!("{tree:?}");

        let test_id2 = EntityId::new(2);
        let first_child = *tree.get_first_child(&test_id2).unwrap();
        assert_eq!(first_child, EntityId::new(3));

        let removed = tree.remove(EntityId::new(3));
        let removed_len = removed.len(&EntityId::new(3));
        let after_remove_len = tree.len(&root);
        assert_eq!(removed_len, 2);
        assert_eq!(after_remove_len, initial_len - removed_len);

        // eprintln!("{removed:?}");
        // eprintln!("{tree:?}");

        // let first_child_after_removal = tree.get_first_child(&test_id2);
    }

    #[test]
    fn remove_sub_tree() {
        let (_, mut tree) = setup_tree(11);
        let root = EntityId::new(0);
        let len = tree.len(&root);
        // eprintln!("{tree:?}");

        let id = EntityId::new(8);
        let removed = tree.remove_subtree(id);
        let removed_len = removed.len();
        // eprintln!("{removed:?}");

        let after_remove_len = tree.len(&root);
        assert_eq!(len, removed_len + after_remove_len);

        tree.insert_subtree(removed, None);
        // eprintln!("{tree:?}");
    }

    #[test]
    fn sibling_test() {
        let (mut manager, mut tree) = setup_tree(11);
        let existing_entity = EntityId::new(6);
        let new_id = manager.create().id;
        let err_add = tree.try_add_sibling(&new_id, existing_entity);
        assert!(err_add.is_err());

        let ok_add = tree.try_add_sibling(&existing_entity, new_id);
        assert!(ok_add.is_ok());
    }

    // #[test]
    // fn stress_test() {
    //     const NUM: usize = 1 << 19;
    //     let (_, tree) = setup_tree(NUM);

    //     let now = std::time::Instant::now();
    //     let n = tree.iter_depth(TestId::root()).count();
    //     eprintln!("depth traverse time for {n} nodes: {:?}", now.elapsed());

    //     let now = std::time::Instant::now();
    //     let n = tree.iter_breadth(TestId::root()).count();
    //     eprintln!("forward breadth traverse time for {n} nodes: {:?}", now.elapsed());

    //     let now = std::time::Instant::now();
    //     let n = tree.iter_breadth(TestId::root()).rev().count();
    //     eprintln!("backward breadth traverse time for {n} nodes: {:?}", now.elapsed());
    // }
}
