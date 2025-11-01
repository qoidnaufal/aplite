use crate::entity::EntityId;
use crate::iterator::{
    TreeChildIter,
    TreeDepthIter,
    TreeAncestryIter,
    TreeNodeIter,
};
use super::node::SubTree;

/// Array based data structure, where the related information is allocated parallel to the main [`EntityId`].
/// This should enable fast and efficient indexing when accessing the data. Internally the data is stored using [`IndexMap`](crate::indexmap::IndexMap).
/// 
/// Another alternative would be to use [`IndexMap`](crate::indexmap::IndexMap) directly and store a custom TreeNode.
/// # Custom Tree Example
/// ```ignore
/// struct CustomTree {
///     storage: IndexMap<TreeNode>
/// }
///
/// struct TreeNode {
///     parent: Option<EntityId>,
///     children: Vec<EntityId>,
/// }
/// ```
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

    #[inline(always)]
    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent, or the next sibling of parent's last child.
    /// If a [`TreeError`] is returned it means that the parent is invalid, usually because you haven't registered it to the tree
    pub fn try_insert(&mut self, id: EntityId, parent: Option<EntityId>) -> Result<(), TreeError> {
        let parent = parent.unwrap_or(EntityId::root());
        if parent.index() > self.parent.len() { return Err(TreeError::InvalidParent) }

        let index = id.index();
        self.resize_if_needed(index);
        self.parent[index] = Some(parent);

        if let Some(first) = self.get_first_child(&parent) {
            let mut current = *first;

            while let Some(next) = self.get_next_sibling(&current) {
                current = *next;
            }

            self.next_sibling[current.index()] = Some(id);
            self.prev_sibling[index] = Some(current);
        } else {
            self.first_child[parent.index()] = Some(id);
        }

        Ok(())
    }

    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent,
    /// or the next sibling of parent's last child.
    pub fn insert(&mut self, id: EntityId, parent: Option<EntityId>) {
        self.try_insert(id, parent).unwrap()
    }

    pub fn insert_subtree(&mut self, subtree: SubTree, parent: Option<EntityId>) {
        self.insert(*subtree.id(), parent);
        subtree.iter_member_ref()
            .for_each(|node_ref| {
                self.insert(*node_ref.entity, node_ref.parent.copied());
            });
    }

    pub fn add_child(&mut self, id: &EntityId, child: EntityId) {
        self.try_add_child(id, child).unwrap()
    }

    pub fn try_add_child(&mut self, id: &EntityId, child: EntityId) -> Result<(), TreeError> {
        if id.index() >= self.parent.len() { return Err(TreeError::InvalidEntity) }

        let child_index = child.index();

        self.resize_if_needed(child_index);

        if let Some(last) = self.get_last_child(id) {
            let mut current = *last;
            while let Some(next) = self.get_next_sibling(&current) {
                current = *next;
            }

            self.parent[child_index] = Some(*id);
            self.next_sibling[current.index()] = Some(*id);
            self.prev_sibling[child_index] = Some(current);
        } else {
            self.first_child[id.index()] = Some(*id);
        }

        Ok(())
    }

    /// Add a sibling to an entity. This will check if the provided entity is a valid one or not.
    /// If the returned result is [`TreeError`], this means the provided entity is either not registered,
    /// or is actually a root. Maybe you want to add a root instead
    pub fn try_add_sibling(&mut self, id: &EntityId, sibling: EntityId) -> Result<(), TreeError> {
        if id.index() >= self.parent.len() { return Err(TreeError::InvalidEntity) }

        let Some(parent) = self.get_parent(id).copied() else { return Err(TreeError::InvalidEntity) };

        let sibling_index = sibling.index();

        self.resize_if_needed(sibling_index);

        let mut current = *id;
        while let Some(next) = self.get_next_sibling(&current) {
            current = *next;
        }

        self.parent[sibling_index] = Some(parent);
        self.next_sibling[current.index()] = Some(sibling);
        self.prev_sibling[sibling_index] = Some(current);

        Ok(())
    }

    /// Add a sibling to an entity. This will check the current sibling of the entity.
    /// If [`None`], immediately sets the next sibling. If [`Some`], loop until find the last sibling.
    pub fn add_sibling(&mut self, id: &EntityId, sibling: EntityId) {
        self.try_add_sibling(id, sibling).unwrap()

        // else {
        //     self.add_root(sibling);
        //     self.next_sibling[entity.index()] = Some(sibling);
        // }
    }

    #[inline(always)]
    /// Currently produces another Tree with the member of the removed entity.
    /// Kinda inefficient if the entity has super big index.
    pub fn remove(&mut self, id: EntityId) -> Self {
        let mut removed_branch = Self::default();
        removed_branch.insert(id, None);

        self.iter_node(&id)
            .for_each(|node| {
                removed_branch.insert(*node.entity, node.parent.copied());
            });

        // shifting
        if let Some(prev) = self.get_prev_sibling(&id).copied() {
            self.next_sibling[prev.index()] = self.get_next_sibling(&id).copied();
        } else if let Some(parent) = self.get_parent(&id).copied() {
            self.first_child[parent.index()] = self.get_next_sibling(&id).copied();
        }

        if let Some(next) = self.get_next_sibling(&id).copied() {
            self.prev_sibling[next.index()] = self.get_prev_sibling(&id).copied();
        }

        let entity_index = id.index();

        self.parent[entity_index] = None;
        self.first_child[entity_index] = None;
        self.next_sibling[entity_index] = None;
        self.prev_sibling[entity_index] = None;

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

        // shifting
        if let Some(prev) = self.get_prev_sibling(&id).copied() {
            self.next_sibling[prev.index()] = self.get_next_sibling(&id).copied();
        } else if let Some(parent) = self.get_parent(&id).copied() {
            self.first_child[parent.index()] = self.get_next_sibling(&id).copied();
        }

        if let Some(next) = self.get_next_sibling(&id).copied() {
            self.prev_sibling[next.index()] = self.get_prev_sibling(&id).copied();
        }

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
        self.iter_depth(&EntityId::root()).count() == 0
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
        fn get_frame<'a>(tree: &Tree, id: &EntityId) -> &'a str {
            match tree.get_next_sibling(id) {
                Some(_) => "├─",
                None => "└─",
            }
        }

        fn recursive_print(tree: &Tree, start: Option<&EntityId>, s: &mut String) {
            match start {
                Some(parent) => {
                    tree.iter_children(parent).for_each(|child| {
                        let ancestor_sibling = tree.ancestors_with_sibling(child);
                        let loc = ancestor_sibling
                            .iter()
                            .enumerate()
                            .map(|(i, val)| val.then_some(i).unwrap_or_default())
                            .max()
                            .unwrap_or_default();

                        let depth = tree.entity_depth(child);
                        let frame = get_frame(tree, child);
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

                        recursive_print(tree, Some(child), s);
                    });
                },
                None => {
                    s.push_str(format!(" > {:?}\n", EntityId::root()).as_str());
                    recursive_print(tree, Some(&EntityId::root()), s);
                },
            }
        }

        let mut s = String::new();
        recursive_print(self, None, &mut s);
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub enum TreeError {
    InvalidParent,
    InvalidEntity,
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
    use crate::{EntityId, IdManager};

    fn setup_tree(num: usize) -> (IdManager, Tree) {
        let mut manager = IdManager::default();
        let mut tree = Tree::with_capacity(num);
        let mut parent = Some(EntityId::root());
        for i in 0..num {
            let id = manager.create();
            tree.insert(id, parent);
            if i > 0 && i % 3 == 0 {
                parent = tree.get_first_child(&EntityId::new(1, 0)).copied();
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

        let ancestor_id = EntityId::new(9, 0);
        let ancestor = tree.get_root(&ancestor_id);

        let test_id_6 = EntityId::new(6, 0);
        let parent = tree.get_parent(&test_id_6);
        let four_is_mem_of_two = tree.is_member_of(&EntityId::new(4, 0), &EntityId::new(2, 0));
        let nine_is_mem_of_two = tree.is_member_of(&EntityId::new(9, 0), &EntityId::new(2, 0));

        let test_id_4 = EntityId::new(4, 0);
        let next_sibling = tree.get_next_sibling(&test_id_4);

        assert_eq!(ancestor, Some(&EntityId::new(0, 0)));
        assert_eq!(parent, Some(&EntityId::new(5, 0)));
        assert_eq!(four_is_mem_of_two, nine_is_mem_of_two);
        assert_eq!(next_sibling, None);
    }

    #[test]
    fn member_depth_test() {
        let (_, tree) = setup_tree(11);
        // eprintln!("{tree:?}");

        let root_children = tree.iter_children(&EntityId::root()).count();
        let all = tree.iter_children(&EntityId::root())
            .map(|id| tree.iter_depth(id).count())
            .sum::<usize>();

        assert_eq!(all + root_children, tree.len(&EntityId::root()));

        let subtree_len = tree.len(&EntityId::new(5, 0));

        assert_eq!(subtree_len, 3);
    }

    #[test]
    fn remove_first_child() {
        let (_, mut tree) = setup_tree(11);
        let initial_len = tree.len(&EntityId::root());

        // eprintln!("{tree:?}");

        let test_id2 = EntityId::new(2, 0);
        let first_child = *tree.get_first_child(&test_id2).unwrap();
        assert_eq!(first_child, EntityId::new(3, 0));

        let removed = tree.remove(EntityId::new(3, 0));
        let removed_len = removed.len(&EntityId::new(3, 0));
        let after_remove_len = tree.len(&EntityId::root());
        assert_eq!(removed_len, 2);
        assert_eq!(after_remove_len, initial_len - removed_len);

        // eprintln!("{removed:?}");
        // eprintln!("{tree:?}");

        // let first_child_after_removal = tree.get_first_child(&test_id2);
    }

    #[test]
    fn remove_sub_tree() {
        let (_, mut tree) = setup_tree(11);
        let root = EntityId::root();
        let len = tree.len(&root);
        // eprintln!("{tree:?}");

        let id = EntityId::new(8, 0);
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
        let existing_entity = EntityId::new(6, 0);
        let new_id = manager.create();
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
