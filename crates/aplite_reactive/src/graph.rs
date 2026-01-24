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

static SCOPE: RwLock<Option<WeakScope>> = RwLock::new(None);

pub struct Scope(Arc<RwLock<ReactiveScope>>);

pub(crate) struct WeakScope(Weak<RwLock<ReactiveScope>>);

struct ReactiveScope {
    parent: Option<WeakScope>,
    children: Vec<WeakScope>,
    node_ids: Vec<SlotId>,
}

impl Scope {
    pub fn new() -> Self {
        let current_scope = SCOPE.read()
            .unwrap()
            .as_ref()
            .map(WeakScope::clone);

        Self(Arc::new_cyclic(|this| {
            if let Some(current) = current_scope.as_ref()
                .and_then(WeakScope::upgrade) {
                current.0.write()
                    .unwrap()
                    .children
                    .push(WeakScope(Weak::clone(this)));
            }

            RwLock::new(ReactiveScope {
                parent: current_scope,
                children: Vec::new(),
                node_ids: Vec::new(),
            })
        }))
    }

    pub(crate) fn downgrade(&self) -> WeakScope {
        WeakScope(Arc::downgrade(&self.0))
    }

    pub fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        let mut lock = SCOPE.write().unwrap();
        let prev = lock.replace(self.downgrade());
        drop(lock);

        let res = f();
        *SCOPE.write().unwrap() = prev;
        res
    }

    pub fn with_cleanup<R>(&self, f: impl FnOnce() -> R) -> R {
        self.cleanup();
        self.with(f)
    }

    pub fn cleanup(&self) {
        let mut lock = self.0.write().unwrap();

        let children = std::mem::take(&mut lock.children);
        let node_ids = std::mem::take(&mut lock.node_ids);

        for child in children {
            if let Some(child) = child.upgrade() {
                child.cleanup()
            }
        }

        ReactiveStorage::with_mut(|graph| {
            for id in node_ids {
                graph.inner.remove(id);
            }
        });
    }

    pub(crate) fn with_current<R>(f: impl FnOnce(&WeakScope) -> R) -> Option<R> {
        let lock = SCOPE.read().unwrap();
        let weak = lock.as_ref().map(Clone::clone);
        drop(lock);
        weak.as_ref().map(f)
    }
}

impl WeakScope {
    pub(crate) fn upgrade(&self) -> Option<Scope> {
        self.0.upgrade().map(Scope)
    }

    fn add_id(&self, id: SlotId) {
        if let Some(scope) = self.upgrade() {
            let mut lock = scope.0.write().unwrap();
            if !lock.node_ids.contains(&id) {
                lock.node_ids.push(id);
            }
        }
    }
}

impl Clone for WeakScope {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl PartialEq for WeakScope {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lock = self.0.read().unwrap();
        let reactive_scope = &*lock;
        reactive_scope.fmt(f)
    }
}

impl std::fmt::Debug for WeakScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("id", &Weak::as_ptr(&self.0).addr())
            .finish()
    }
}

impl std::fmt::Debug for ReactiveScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("parent", &self.parent)
            .field("children", &self.children
                .iter()
                .map(|c| c.upgrade())
                .collect::<Vec<_>>()
            )
            .field("ids", &self.node_ids)
            .finish()
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
static STORAGE: OnceLock<RwLock<ReactiveStorage>> = OnceLock::new();

#[derive(Default)]
pub struct ReactiveStorage {
    pub(crate) inner: SlotMap<Box<dyn Any + Send + Sync>>,
}

unsafe impl Send for ReactiveStorage {}
unsafe impl Sync for ReactiveStorage {}

impl ReactiveStorage {
    #[inline(always)]
    fn read<'a>() -> RwLockReadGuard<'a, ReactiveStorage> {
        STORAGE.get_or_init(Default::default).read().unwrap()
    }

    #[inline(always)]
    fn try_read<'a>() -> Option<RwLockReadGuard<'a, ReactiveStorage>> {
        STORAGE.get_or_init(Default::default).read().ok()
    }

    #[inline(always)]
    fn write<'a>() -> RwLockWriteGuard<'a, ReactiveStorage> {
        STORAGE.get_or_init(Default::default).write().unwrap()
    }

    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut ReactiveStorage) -> R,
        R: 'static,
    {
        f(&mut Self::write())
    }

    pub fn with_downcast<R, F, U>(node: &Node<R>, f: F) -> U
    where
        R: 'static,
        F: FnOnce(&R) -> U,
    {
        Self::read().inner.get(&node.id)
            .and_then(|any| any.downcast_ref::<R>())
            .map(f)
            .unwrap()
    }

    pub fn try_with_downcast<R, F, U>(node: &Node<R>, f: F) -> Option<U>
    where
        R: 'static,
        F: FnOnce(&R) -> Option<U>,
    {
        Self::try_read().and_then(|guard| {
            guard.inner.get(&node.id)
                .and_then(|any| any.downcast_ref::<R>())
                .and_then(f)
        })
    }

    pub fn map_with_downcast<F, R, U>(node: &Node<R>, f: F) -> Option<U>
    where
        F: FnOnce(&R) -> U,
        R: 'static,
    {
        Self::try_read().and_then(|guard| {
            guard.inner.get(&node.id)
                .and_then(|any| any.downcast_ref::<R>())
                .map(f)
        })
    }

    pub fn insert<R: Send + Sync + 'static>(r: R) -> Node<R> {
        let mut storage = Self::write();
        let id = storage.inner.insert(Box::new(r));

        Scope::with_current(|weak| weak.add_id(id));

        Node { id, marker: PhantomData }
    }

    pub fn remove<R: 'static>(node: Node<R>) {
        let mut storage = Self::write();
        storage.inner.remove(node.id);
    }

    pub fn is_removed<R>(node: &Node<R>) -> bool {
        let storage = Self::read();
        storage.inner.get(&node.id).is_none()
    }
}

/*
#########################################################
#
# Observer
#
#########################################################
*/

static OBSERVER: RwLock<Observer> = RwLock::new(Observer(None));

pub struct Observer(Option<AnySubscriber>);

impl Observer {
    pub(crate) fn with<U>(f: impl FnOnce(Option<&AnySubscriber>) -> U) -> U {
        f(OBSERVER.read().unwrap().0.as_ref())
    }

    pub(crate) fn swap_observer(subscriber: Option<AnySubscriber>) -> Option<AnySubscriber> {
        let mut current = OBSERVER.write().unwrap();
        let prev = current.0.take();
        current.0 = subscriber;
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

pub struct Node<R> {
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

