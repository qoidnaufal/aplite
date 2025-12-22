use std::marker::PhantomData;
use std::any::Any;
use std::sync::{
    Arc,
    Weak,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
    OnceLock
};

use aplite_storage::{SlotMap, SlotId};
use crate::subscriber::AnySubscriber;

/*
#########################################################
#
# Scope
#
#########################################################
*/

pub(crate) struct ReactiveScope {
    parent: Option<WeakScope>,
    storage: SlotMap<Box<dyn Any + Send + Sync>>,
    children: Vec<WeakScope>,
    node_ids: Vec<SlotId>,
    cleanups: Vec<Box<dyn FnOnce() + Send + Sync>>,
    paused: bool,
}

impl ReactiveScope {
    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn cleanup(&mut self) {
        let cleanups = std::mem::take(&mut self.cleanups);
        let children = std::mem::take(&mut self.children);
        let node_ids = std::mem::take(&mut self.node_ids);

        for cleanup_fn in cleanups {
            cleanup_fn()
        }

        for child in children {
            if let Some(child) = child.upgrade() {
                child.cleanup()
            }
        }

        Graph::with_mut(|graph| {
            for id in node_ids {
                graph.storage.remove(id);
            }
        });
    }
}

static SCOPE: RwLock<Option<WeakScope>> = RwLock::new(None);

pub struct Scope(Arc<RwLock<ReactiveScope>>);

impl Scope {
    pub fn new() -> Self {
        let parent = SCOPE.read().unwrap().as_ref().map(WeakScope::clone);

        Self(Arc::new_cyclic(|weak| {
            if let Some(parent) = parent.as_ref().and_then(WeakScope::upgrade) {
                parent.0.write()
                    .unwrap()
                    .children
                    .push(WeakScope(weak.clone()));
            }

            RwLock::new(ReactiveScope {
                parent: parent.clone(),
                children: Vec::new(),
                storage: SlotMap::default(),
                node_ids: Vec::new(),
                cleanups: Vec::new(),
                paused: false,
            })
        }))
    }

    pub(crate) fn downgrade(&self) -> WeakScope {
        WeakScope(Arc::downgrade(&self.0))
    }

    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        let prev = SCOPE.write().unwrap().replace(self.downgrade());

        let res = f();

        *SCOPE.write().unwrap() = prev;

        res
    }

    pub fn with_cleanup<R>(&self, f: impl FnOnce() -> R) -> R {
        self.cleanup();
        self.with(f)
    }

    pub(crate) fn cleanup(&self) {
        self.0.write().unwrap().cleanup()
    }

    pub(crate) fn paused(&self) -> bool {
        self.0.read().unwrap().paused
    }
}

pub struct WeakScope(Weak<RwLock<ReactiveScope>>);

impl Clone for WeakScope {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl WeakScope {
    #[inline(always)]
    fn upgrade(&self) -> Option<Scope> {
        self.0.upgrade().map(Scope)
    }
}

/*
#########################################################
#
# ReactiveNodeStorage
#
#########################################################
*/

// had to use OnceLock because we don't know yet which reactive node will initialize this first
static STORAGE: OnceLock<RwLock<Graph>> = OnceLock::new();

#[derive(Default)]
pub(crate) struct Graph {
    pub(crate) storage: SlotMap<Box<dyn Any + Send + Sync>>,
}

unsafe impl Send for Graph {}
unsafe impl Sync for Graph {}

impl Graph {
    pub(crate) fn insert<R: Send + Sync + 'static>(r: R) -> Node<R> {
        let mut graph = Self::write();
        let id = graph.storage.insert(Box::new(r));
        Node { id, marker: PhantomData }
    }

    #[inline(always)]
    fn read<'a>() -> RwLockReadGuard<'a, Graph> {
        STORAGE.get_or_init(Default::default).read().unwrap()
    }

    #[inline(always)]
    fn write<'a>() -> RwLockWriteGuard<'a, Graph> {
        STORAGE.get_or_init(Default::default).write().unwrap()
    }

    // #[inline(always)]
    // pub(crate) fn with<U>(f: impl FnOnce(&Graph) -> U) -> U {
    //     f(&Self::read())
    // }

    #[inline(always)]
    pub(crate) fn with_mut<U>(f: impl FnOnce(&mut Graph) -> U) -> U {
        f(&mut Self::write())
    }

    pub(crate) fn with_downcast<R, F, U>(node: &Node<R>, f: F) -> U
    where
        R: 'static,
        F: FnOnce(&R) -> U,
    {
        let graph = Self::read();
        let r = graph
            .storage
            .get(&node.id)
            .and_then(|any| any.as_ref().downcast_ref::<R>())
            .unwrap();
        f(r)
    }

    pub(crate) fn try_with_downcast<R, F, U>(node: &Node<R>, f: F) -> Option<U>
    where
        R: 'static,
        F: FnOnce(Option<&R>) -> Option<U>,
    {
        Self::read()
            .storage
            .get(&node.id)
            .and_then(|any| f(any.as_ref().downcast_ref::<R>()))
    }

    pub(crate) fn remove<R>(node: &Node<R>) {
        let mut graph = Self::write();
        graph.storage.remove(node.id);
    }

    pub(crate) fn is_removed<R>(node: &Node<R>) -> bool {
        let graph = Self::read();
        graph.storage.get(&node.id).is_none()
    }
}

/*
#########################################################
#
# Observer
#
#########################################################
*/

static OBSERVER: RwLock<Option<AnySubscriber>> = RwLock::new(None);

pub(crate) struct Observer;

impl Observer {
    #[inline(always)]
    pub(crate) fn with<U>(f: impl FnOnce(Option<&AnySubscriber>) -> U) -> U {
        f(OBSERVER.read().unwrap().as_ref())
    }

    pub(crate) fn swap_observer(subscriber: Option<AnySubscriber>) -> Option<AnySubscriber> {
        let mut current = OBSERVER.write().unwrap();
        let prev = current.take();
        *current = subscriber;
        prev
    }
}

/*
#########################################################
#
# Node
#
#########################################################
*/

pub(crate) struct Node<R> {
    pub(crate) id: SlotId,
    marker: PhantomData<R>,
}

impl<R> Clone for Node<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> PartialEq for Node<R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<R> PartialOrd for Node<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R> Ord for Node<R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<R> Copy for Node<R> {}
impl<R> Eq for Node<R> {}

impl<R> std::hash::Hash for Node<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<R> std::fmt::Debug for Node<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<R>())
            .field("id", &self.id)
            .finish()
    }
}
