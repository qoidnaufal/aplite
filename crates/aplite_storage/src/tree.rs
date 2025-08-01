use crate::entity::Entity;
use crate::iterator::{TreeIterator, NodeRef};
use crate::index_map::IndexMap;

/// Array based data structure, where the related information is allocated parallel to the main [`Entity`].
/// This should enable fast and efficient indexing when accessing the data.
/// Internally the data is stored using [`IndexMap`].
pub struct Tree<E: Entity, T> {
    pub(crate) data: IndexMap<E, T>,
    pub(crate) parent: Vec<Option<E>>,
    pub(crate) first_child: Vec<Option<E>>,
    pub(crate) next_sibling: Vec<Option<E>>,
}

impl<E: Entity, T> Default for Tree<E, T> {
    fn default() -> Self {
        Self {
            data: IndexMap::new(),
            parent: Vec::new(),
            first_child: Vec::new(),
            next_sibling: Vec::new(),
        }
    }
}

impl<E: Entity, T> Tree<E, T> {
    /// This will create a default [`Tree`] without preallocating an initial capacity.
    /// If you want to specify the initial capacity, use [`Tree::with_capacity()`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`Tree`] with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: IndexMap::with_capacity(capacity),
            parent: Vec::with_capacity(capacity),
            first_child: Vec::with_capacity(capacity),
            next_sibling: Vec::with_capacity(capacity),
        }
    }

    /// This method will create a new [`Entity`](Entity), and immediately insert it into the tree.
    /// Doesn't calculate the location of the created entity.
    /// You can later add children, next siblings, or set the parent to this entity
    pub fn insert(&mut self, data: T) -> E {
        let entity = self.data.insert(data);
        self.first_child.push(None);
        self.next_sibling.push(None);
        self.parent.push(None);
        entity
    }

    pub fn replace(&mut self, entity: &E, data: T) -> Option<T> {
        self.data.replace(entity, data)
    }

    pub fn get(&self, entity: &E) -> Option<&T> {
        self.data.get(entity)
    }

    pub fn get_mut(&mut self, entity: &E) -> Option<&mut T> {
        self.data.get_mut(entity)
    }

    pub fn remove(&mut self, entity: E) -> Vec<E> {
        let mut to_remove = vec![entity];
        let mut current = entity;

        while let Some(children) = self.get_all_children(&current) {
            children.iter().for_each(|child| current = *child);
            to_remove.extend_from_slice(&children);
        }

        // shifting
        if let Some(prev) = self.get_prev_sibling(&entity).copied() {
            self.next_sibling[prev.index()] = self.get_next_sibling(&entity).copied();
        } else if let Some(parent) = self.get_parent(&entity).copied() {
            self.first_child[parent.index()] = self.get_next_sibling(&entity).copied();
        }
        
        to_remove
            .iter()
            .for_each(|entity| {
                self.data.remove(entity);
                self.parent[entity.index()] = None;
            });
        to_remove
    }

    /// Adding an entity to be the child of a parent.
    /// This will calculate if it's the first child of the parent,
    /// or the next sibling of parent's last child.
    pub fn add_child(&mut self, parent: &E, child: E) {
        match self.get_last_child(parent).copied() {
            Some(last) => self.add_sibling_with_parent(Some(*parent), &last, child),
            None => self.add_first_child(parent, child),
        }
    }

    /// Add a sibling to an entity. This will check the current sibling of the entity.
    /// If [`None`], immediately sets the next sibling. If [`Some`], loop until find the last sibling.
    pub fn add_sibling(&mut self, entity: &E, sibling: E) {
        let parent = self.get_parent(entity).copied();
        self.add_sibling_with_parent(parent, entity, sibling);
    }

    #[inline(always)]
    fn set_parent(&mut self, entity: &E, parent: Option<E>) {
        self.parent[entity.index()] = parent;
    }

    #[inline(always)]
    fn add_first_child(&mut self, entity: &E, child: E) {
        self.set_parent(&child, Some(*entity));
        self.first_child[entity.index()] = Some(child);

        let mut current = child;
        while let Some(sibling) = self.get_next_sibling(&current).copied() {
            self.set_parent(&sibling, Some(*entity));
            current = sibling;
        }
    }

    #[inline(always)]
    fn add_sibling_with_parent(
        &mut self,
        parent: Option<E>,
        entity: &E,
        next_sibling: E,
    ) {
        self.set_parent(&next_sibling, parent);
        let mut current = *entity;
        while let Some(sibling) = self.get_next_sibling(&current).copied() {
            current = sibling;
        }
        self.next_sibling[current.index()] = Some(next_sibling);
    }

    pub fn get_all_entities(&self) -> Vec<E> {
        self.data
            .iter()
            .map(|(entity, _)| entity)
            .collect()
    }

    /// get all the entities which has no parent
    pub fn get_all_roots(&self) -> Vec<E> {
        self.data
            .iter()
            .filter_map(|(e, _)| {
                self.get_parent(&e)
                    .is_none()
                    .then_some(e)
            })
            .collect()

    }

    /// get the root of an entity
    pub fn get_root<'a>(&'a self, entity: &'a E) -> Option<&'a E> {
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

    /// the distance of an entity from the root
    pub fn depth(&self, entity: &E) -> usize {
        let mut current = entity;
        let mut depth = 0;
        while let Some(parent) = self.get_parent(current) {
            depth += 1;
            current = parent;
        }
        depth
    }

    pub fn ancestors_with_sibling(&self, entity: &E) -> Vec<bool> {
        let mut current = entity;
        let mut loc = vec![];
        while let Some(parent) = self.get_parent(current) {
            loc.push(self.get_next_sibling(parent).is_some());
            current = parent;
        }
        loc.reverse();
        loc
    }

    pub fn get_parent(&self, entity: &E) -> Option<&E> {
        self.parent[entity.index()].as_ref()
    }

    pub fn get_parent_mut(&mut self, entity: &E) -> Option<&mut E> {
        self.parent[entity.index()].as_mut()
    }

    pub fn get_first_child(&self, entity: &E) -> Option<&E> {
        self.first_child[entity.index()].as_ref()
    }

    pub fn get_last_child(&self, entity: &E) -> Option<&E> {
        let maybe_first = self.get_first_child(entity);
        if let Some(first) = maybe_first {
            let mut last = first;
            while let Some(next) = self.get_next_sibling(last) {
                last = next;
            }
            Some(last)
        } else {
            None
        }
    }

    pub fn get_next_sibling(&self, entity: &E) -> Option<&E> {
        self.next_sibling[entity.index()].as_ref()
    }

    pub fn get_prev_sibling(&self, entity: &E) -> Option<&E> {
        if let Some(parent) = self.get_parent(entity) {
            let mut first = self.get_first_child(parent).unwrap();
            while let Some(next) = self.get_next_sibling(first) {
                if next == entity {
                    return Some(first);
                }
                first = next;
            }
            None
        } else {
            None
        }
    }

    pub fn get_all_children(&self, entity: &E) -> Option<Vec<E>> {
        self.get_first_child(entity).map(|first| {
            let mut curr = first;
            let mut children = vec![*curr];
            while let Some(next) = self.get_next_sibling(curr) {
                children.push(*next);
                curr = next;
            }
            children
        })
    }

    pub fn get_all_members_of(&self, entity: &E) -> Vec<E> {
        let mut members = vec![];
        if let Some(children) = self.get_all_children(entity) {
            children.iter().for_each(|id| {
                members.push(*id);
                let inner = self.get_all_members_of(id);
                members.extend_from_slice(&inner);
            });
        }
        members
    }

    pub fn is_member_of(&self, entity: &E, ancestor: &E) -> bool {
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
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get_node_ref<'a>(&'a self, entity: &'a E) -> Option<NodeRef<'a, E, T>> {
        self.get(entity)
            .map(|data| {
                NodeRef::new(self, *entity, data)
            })
    }

    pub fn contains(&self, entity: &E) -> bool {
        self.data.contains(entity)
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.parent.clear();
        self.first_child.clear();
        self.next_sibling.clear();
    }

    pub fn iter(&self) -> TreeIterator<'_, E, T> { self.into_iter() }
}

impl<E: Entity, T> std::fmt::Debug for Tree<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn get_frame<'a, E: Entity, T>(tree: &'a Tree<E, T>, entity: &'a E) -> &'a str {
            match tree.get_next_sibling(entity) {
                Some(_) => "├─",
                None => "└─",
            }
        }

        fn recursive_print<E: Entity, T>(tree: &Tree<E, T>, start: Option<&E>, s: &mut String) {
            match start {
                Some(parent) => {
                    if let Some(children) = tree.get_all_children(parent) {
                        children.iter().for_each(|child| {
                            let ancestor_sibling = tree.ancestors_with_sibling(child);
                            let loc = ancestor_sibling
                                .iter()
                                .enumerate()
                                .map(|(i, &val)| if val { i } else { 0 })
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
                    }
                },
                None => {
                    let roots = tree.get_all_roots();
                    roots
                        .iter()
                        .for_each(|root| {
                            s.push_str(format!(">> {root:?} : Root\n").as_str());
                            if tree.get_first_child(root).is_some() {
                                recursive_print(tree, Some(root), s);
                            }
                        });
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
    use crate::entity;

    entity! { TestId }

    fn setup_tree() -> Tree<TestId, ()> {
        let mut tree: Tree<TestId, ()> = Tree::with_capacity(10);
        let mut parent = None;
        for i in 0..11 {
            let id = tree.insert(());
            if let Some(parent) = parent.as_ref() {
                tree.add_child(parent, id);
            }
            if i > 0 && i % 3 == 0 {
                parent = tree.get_first_child(&TestId(1, 0)).map(|e| *e);
            } else {
                parent = Some(id);
            }
        }
        tree
    }

    #[test]
    fn tree_test() {
        let tree = setup_tree();
        eprintln!("{tree:?}");
        eprintln!("{:?}", tree.data);

        let ancestor = tree.get_root(&TestId(9, 0));
        let parent = tree.get_parent(&TestId(6, 0));
        let four_is_mem_of_two = tree.is_member_of(&TestId(4, 0), &TestId(2, 0));
        let nine_is_mem_of_two = tree.is_member_of(&TestId(9, 0), &TestId(2, 0));
        let next_sibling = tree.get_next_sibling(&TestId(4, 0));

        assert_eq!(ancestor, Some(&TestId(0, 0)));
        assert_eq!(parent, Some(&TestId(5, 0)));
        assert_eq!(four_is_mem_of_two, nine_is_mem_of_two);
        assert_eq!(next_sibling, Some(&TestId(7, 0)));
    }

    #[test]
    fn remove_first_child() {
        let mut tree = setup_tree();

        let first_child = tree.get_first_child(&TestId(2, 0)).copied().unwrap();
        let next_sibling = tree.get_next_sibling(&first_child).copied();
        assert_eq!(first_child, TestId(3, 0));

        let _removed = tree.remove(TestId(3, 0));

        let first_child_after_removal = tree.get_first_child(&TestId(2, 0)).copied();
        assert_eq!(first_child_after_removal, next_sibling);

        // eprintln!("{tree:?}");
        // eprintln!("remaining {} > {:#?}", tree.manager.len(), tree.manager);
        // eprintln!("removed > {removed:?}");
    }

    #[test]
    fn remove_middle_child() {
        let mut tree = setup_tree();

        let sibling_before_removal = tree.get_next_sibling(&TestId(4, 0)).copied();
        let _removed = tree.remove(TestId(4, 0));
        let sibling_after_removal = tree.get_next_sibling(&TestId(3, 0)).copied();
        assert_eq!(sibling_before_removal, sibling_after_removal);

        // eprintln!("{tree:?}");
        // eprintln!("remaining {} > {:#?}", tree.manager.len(), tree.manager);
        // eprintln!("removed > {removed:?}");
    }

    #[test]
    fn insert_after_remove() {
        let mut tree = setup_tree();

        let _ = tree.remove(TestId(4, 0));
        let reuse = tree.insert(());

        tree.add_child(&TestId(7, 0), reuse);
        assert_eq!(&TestId(7, 0), tree.get_parent(&reuse).unwrap());

        eprintln!("{tree:?}");
    }
}
