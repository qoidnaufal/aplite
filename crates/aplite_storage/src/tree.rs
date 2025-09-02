use crate::entity::Entity;
use crate::iterator::{
    TreeChildIter,
    TreeMemberDepthIter,
    TreeMemberBreadthIter,
    TreeAncestryIter,
    TreeNodeIter,
};

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

/// Array based data structure, where the related information is allocated parallel to the main [`Entity`].
/// This should enable fast and efficient indexing when accessing the data. Internally the data is stored using [`IndexMap`].
/// 
/// Another alternative would be to use [`IndexMap`] directly and store a custom TreeNode.
/// # Custom Tree Example
/// ```ignore
/// struct CustomTree {
///     storage: IndexMap<UniqueId, TreeNode>
/// }
///
/// struct TreeNode {
///     parent: Option<UniqueId>,
///     children: Vec<UniqueId>,
/// }
/// ```
pub struct Tree<E: Entity> {
    pub(crate) parent: Vec<Option<E>>,
    pub(crate) first_child: Vec<Option<E>>,
    pub(crate) next_sibling: Vec<Option<E>>,
    pub(crate) prev_sibling: Vec<Option<E>>,
}

impl<E: Entity> Default for Tree<E> {
    /// This will create a default [`Tree`] without preallocating an initial capacity.
    /// If you want to specify the initial capacity, use [`Tree::with_capacity()`]
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<E: Entity> Tree<E> {
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

    pub fn get_initial_root(&self) -> E {
        self.first_child[0]
            .and_then(|fc| self.parent[fc.index()])
            .expect("A root entity should have been initialized first")
    }

    /// get the root of an entity
    pub fn get_root(&self, entity: E) -> Option<E> {
        let mut current = entity;
        while let Some(parent) = self.get_parent(current) {
            current = parent;
        }
        if current == entity {
            None
        } else {
            Some(current)
        }
    }

    #[inline(always)]
    pub fn get_parent(&self, entity: E) -> Option<E> {
        self.parent[entity.index()]
    }

    #[inline(always)]
    pub fn get_first_child(&self, entity: E) -> Option<E> {
        self.first_child[entity.index()]
    }

    #[inline(always)]
    pub fn get_last_child(&self, entity: E) -> Option<E> {
        let Some(first) = self.get_first_child(entity) else { return None };
        let mut last = first;
        while let Some(next) = self.get_next_sibling(last) {
            last = next;
        }
        Some(last)
    }

    #[inline(always)]
    pub fn get_next_sibling(&self, entity: E) -> Option<E> {
        self.next_sibling[entity.index()]
    }

    #[inline(always)]
    pub fn get_prev_sibling(&self, entity: E) -> Option<E> {
        self.prev_sibling[entity.index()]
        // if let Some(parent) = self.get_parent(entity) {
        //     let mut first = self.get_first_child(parent).unwrap();
        //     while let Some(next) = self.get_next_sibling(first) {
        //         if next == entity {
        //             return Some(first);
        //         }
        //         first = next;
        //     }
        //     None
        // } else {
        //     None
        // }
    }

    /// This method will create an allocation,
    /// If you want to avoid unnecessary allocation use [`iter_children`](Self::iter_children)
    #[inline(always)]
    pub fn get_all_children(&self, entity: E) -> Vec<E> {
        self.iter_children(entity).collect()
    }

    #[inline(always)]
    fn resize(&mut self, index: usize) {
        self.parent.resize(index + 1, None);
        self.first_child.resize(index + 1, None);
        self.next_sibling.resize(index + 1, None);
        self.prev_sibling.resize(index + 1, None);
    }

    #[inline(always)]
    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent, or the next sibling of parent's last child.
    /// If a [`TreeError`] is returned it means that the parent is invalid, usually because you haven't registered it to the tree
    pub fn try_insert(&mut self, entity: E, parent: E) -> Result<(), TreeError> {
        if parent.index() > self.parent.len() { return Err(TreeError::InvalidParent) }

        let index = entity.index();

        if index >= self.parent.len() {
            self.resize(index);
        }

        self.parent[index] = Some(parent);
        self.first_child[index] = None;
        self.next_sibling[index] = None;
        self.prev_sibling[index] = None;

        if let Some(first) = self.get_first_child(parent) {
            let mut current = first;

            while let Some(next) = self.get_next_sibling(current) {
                current = next;
            }

            self.next_sibling[current.index()] = Some(entity);
            self.prev_sibling[index] = Some(current);
        } else {
            self.first_child[parent.index()] = Some(entity);
        }

        Ok(())
    }

    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent,
    /// or the next sibling of parent's last child.
    pub fn insert(&mut self, entity: E, parent: E) {
        self.try_insert(entity, parent).unwrap()
    }

    /// Adding an entity to be the child of the root.
    /// This will calculate if it's the first child of the root,
    /// or the next sibling of rootparent's last child.
    pub fn insert_as_parent(&mut self, entity: E) {
        self.try_insert(entity, E::root()).unwrap()
    }

    /// Add a sibling to an entity. This will check if the provided entity is a valid one or not.
    /// If the returned result is [`TreeError`], this means the provided entity is either not registered,
    /// or is actually a root. Maybe you want to add a root instead
    pub fn try_add_sibling(&mut self, entity: E, sibling: E) -> Result<(), TreeError> {
        if entity.index() >= self.parent.len() { return Err(TreeError::InvalidEntity) }

        let Some(parent) = self.get_parent(entity) else { return Err(TreeError::InvalidEntity) };

        let sibling_index = sibling.index();

        if sibling_index >= self.parent.len() {
            self.resize(sibling_index);
        }

        let mut current = entity;
        while let Some(next) = self.get_next_sibling(current) {
            current = next;
        }

        self.parent[sibling_index] = Some(parent);
        self.next_sibling[current.index()] = Some(sibling);
        self.prev_sibling[sibling_index] = Some(current);

        Ok(())
    }

    /// Add a sibling to an entity. This will check the current sibling of the entity.
    /// If [`None`], immediately sets the next sibling. If [`Some`], loop until find the last sibling.
    pub fn add_sibling(&mut self, entity: E, sibling: E) {
        self.try_add_sibling(entity, sibling).unwrap()

        // else {
        //     self.add_root(sibling);
        //     self.next_sibling[entity.index()] = Some(sibling);
        // }
    }

    #[inline(always)]
    pub fn remove(&mut self, entity: E) -> Vec<E> {
        let mut to_remove = vec![entity];

        // maybe inefficient, should just use member iterator?
        let mut current = entity;
        while let Some(first_child) = self.get_first_child(current) {
            to_remove.push(first_child);

            let mut child = first_child;
            while let Some(next) = self.get_next_sibling(child) {
                to_remove.push(next);
                child = next;
            }

            current = first_child;
        }

        // shifting
        if let Some(prev) = self.get_prev_sibling(entity) {
            self.next_sibling[prev.index()] = self.get_next_sibling(entity);
        } else if let Some(parent) = self.get_parent(entity) {
            self.first_child[parent.index()] = self.get_next_sibling(entity);
        }
        
        to_remove
            .iter()
            .for_each(|entity| {
                self.parent[entity.index()] = None;
            });

        to_remove
    }

    /// the distance of an entity from the root
    pub fn depth(&self, entity: E) -> usize {
        let mut current = entity;
        let mut depth = 0;
        while let Some(parent) = self.get_parent(current) {
            depth += 1;
            current = parent;
        }
        depth
    }

    fn ancestors_with_sibling(&self, entity: E) -> Vec<bool> {
        let mut current = entity;
        let mut loc = vec![];
        while let Some(parent) = self.get_parent(current) {
            loc.push(self.get_next_sibling(parent).is_some());
            current = parent;
        }
        loc.reverse();
        loc
    }

    pub fn is_member_of(&self, entity: E, ancestor: E) -> bool {
        if self.get_first_child(ancestor).is_none() {
            return false
        }
        let mut check = entity;
        while let Some(parent) = self.get_parent(check) {
            if parent == ancestor {
                return true;
            }
            check = parent
        }
        check == ancestor
    }

    pub fn len(&self) -> usize {
        self.parent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    pub fn contains(&self, entity: E) -> bool {
        entity.index() <= self.parent.len()
        && (
            self.parent.contains(&Some(entity))
            || self.first_child.contains(&Some(entity))
            || self.next_sibling.contains(&Some(entity))
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
    pub fn iter_children(&self, entity: E) -> TreeChildIter<'_, E> {
        TreeChildIter::new(self, entity)
    }

    /// iterate the members of the entity
    pub fn iter_member_depth(&self, entity: E) -> TreeMemberDepthIter<'_, E> {
        TreeMemberDepthIter::new(self, entity)
    }

    /// iterate the members of the entity
    pub fn iter_member_breadth(&self, entity: E) -> TreeMemberBreadthIter<'_, E> {
        TreeMemberBreadthIter::new(self, entity)
    }

    /// iterate the entity's parent upward
    pub fn iter_ancestry(&self, entity: E) -> TreeAncestryIter<'_, E> {
        TreeAncestryIter::new(self, entity)
    }

    /// iterate the member of the iterator and map it to a [`NodeRef`](crate::iterator::NodeRef)
    pub fn iter_node(&self, entity: E) -> TreeNodeIter<'_, E> {
        TreeNodeIter::new(self, entity)
    }
}

// FIXME: there are two spot which created unnecessary allocation on get_all_children + get_all_roots
impl<E: Entity> std::fmt::Debug for Tree<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn get_frame<'a, E: Entity>(tree: &'a Tree<E>, entity: E) -> &'a str {
            match tree.get_next_sibling(entity) {
                Some(_) => "├─",
                None => "└─",
            }
        }

        fn recursive_print<E: Entity>(tree: &Tree<E>, start: Option<E>, s: &mut String) {
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

                        let depth = tree.depth(child);
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
                    s.push_str(format!(" > {:?}\n", E::root()).as_str());
                    recursive_print(tree, Some(E::root()), s);
                },
            }
        }

        let mut s = String::new();
        recursive_print(self, None, &mut s);
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tree_test {
    use super::*;
    use crate::{Entity, EntityManager};
    use crate::create_entity;

    create_entity! { TestId }

    fn setup_tree() -> (EntityManager<TestId>, Tree<TestId>) {
        let mut manager = EntityManager::<TestId>::default();
        let mut tree: Tree<TestId> = Tree::with_capacity(11);
        let mut parent = Some(TestId::root());
        for i in 0..11 {
            let id = manager.create();
            if let Some(parent) = parent {
                tree.insert(id, parent);
            }
            if i > 0 && i % 3 == 0 {
                parent = tree.get_first_child(TestId::new(1, 0));
            } else {
                parent = Some(id);
            }
        }

        (manager, tree)
    }

    #[test]
    fn tree_test() {
        let (_, tree) = setup_tree();
        // eprintln!("{tree:?}");
        // eprintln!("{:?}", tree.parent);

        let ancestor_id = TestId::new(9, 0);
        let ancestor = tree.get_root(ancestor_id);

        let test_id_6 = TestId::new(6, 0);
        let parent = tree.get_parent(test_id_6);
        let four_is_mem_of_two = tree.is_member_of(TestId::new(4, 0), TestId::new(2, 0));
        let nine_is_mem_of_two = tree.is_member_of(TestId::new(9, 0), TestId::new(2, 0));

        let test_id_4 = TestId::new(4, 0);
        let next_sibling = tree.get_next_sibling(test_id_4);

        assert_eq!(ancestor, Some(TestId::new(0, 0)));
        assert_eq!(parent, Some(TestId::new(5, 0)));
        assert_eq!(four_is_mem_of_two, nine_is_mem_of_two);
        assert_eq!(next_sibling, None);
    }

    #[test]
    fn member_depth_test() {
        let (_, tree) = setup_tree();
        // eprintln!("{tree:?}");

        let root_children = tree.iter_children(TestId::root()).count();
        let all = tree.iter_children(TestId::root())
            .map(|id| tree.iter_member_depth(id).count())
            .sum::<usize>();

        assert_eq!(all + root_children, tree.len() - root_children);

        let member_of_5 = tree.iter_member_depth(TestId(5));
        let count = member_of_5.count();

        assert_eq!(count, 2);
    }

    #[test]
    fn member_breadth_test() {
        let (mut manager, mut tree) = setup_tree();
        let id12 = manager.create();
        tree.insert(id12, TestId(10));
        let root = tree.get_initial_root();
        let horizontal_member = tree.iter_member_breadth(root);

        assert_eq!(horizontal_member.last(), Some(id12));
        eprintln!("{tree:?}");
    }

    #[test]
    fn remove_first_child() {
        let (_, mut tree) = setup_tree();

        let first_child = tree.get_first_child(TestId::new(2, 0)).unwrap();
        let next_sibling = tree.get_next_sibling(first_child);
        assert_eq!(first_child, TestId::new(3, 0));

        let removed = tree.remove(TestId::new(3, 0));
        assert_eq!(removed.len(), 2);

        let first_child_after_removal = tree.get_first_child(TestId::new(2, 0));
        assert_eq!(first_child_after_removal, next_sibling);

        // eprintln!("{tree:?}");
        // eprintln!("remaining {} > {:#?}", tree.manager.len(), tree.manager);
        // eprintln!("removed > {removed:?}");
    }

    #[test]
    fn remove_middle_child() {
        let (_, mut tree) = setup_tree();

        let sibling_before_removal = tree.get_next_sibling(TestId::new(5, 0));
        assert!(sibling_before_removal.is_some_and(|id| tree.contains(id)));

        let removed = tree.remove(TestId::new(5, 0));
        assert!(!tree.contains(TestId::new(5, 0)));
        let sibling_after_removal = tree.get_next_sibling(TestId::new(3, 0));
        assert_eq!(sibling_before_removal, sibling_after_removal);
        assert_eq!(removed.len(), 3);

        // eprintln!("{tree:?}");
        // eprintln!("remaining {}", tree.len());
        // eprintln!("removed > {removed:?}");
    }

    #[test]
    fn sibling_test() {
        let (mut manager, mut tree) = setup_tree();
        let existing_entity = TestId(6);
        let new_id = manager.create();
        let err_add = tree.try_add_sibling(new_id, existing_entity);
        assert!(err_add.is_err());

        let ok_add = tree.try_add_sibling(existing_entity, new_id);
        assert!(ok_add.is_ok());
        // eprintln!("{tree:?}");
    }

    // #[test]
    // fn many_insertion() {
    //     const NUM: usize = 1 << 16;
    //     let mut manager = EntityManager::<TestId>::with_capacity(NUM);
    //     let mut tree: Tree<TestId> = Tree::with_capacity(NUM);
    //     let root = manager.create();
    //     tree.add_root(root);

    //     let mut parent = root;
    //     let now = std::time::Instant::now();
    //     for i in 1..NUM {
    //         let id = manager.create();
    //         if i % 2 == 0 {
    //             tree.add_child(root, id);
    //         } else {
    //             tree.add_child(parent, id);
    //         }
    //         parent = id;
    //     }

    //     assert_eq!(NUM, tree.len());
    //     eprintln!("time needed to create: {} nodes > {:?}", tree.len(), now.elapsed());
    // }
}
