use std::marker::PhantomData;
use std::sync::{Arc, RwLock, OnceLock};
use std::any::Any;

use aplite_storage::IndexMap;
use aplite_macro::entity;

use crate::subscriber::AnySubscriber;
use crate::reactive_traits::*;

static GRAPH: OnceLock<Arc<RwLock<ReactiveGraph>>> = OnceLock::new();

type Storage = IndexMap<ReactiveId, Box<dyn Any + Send + Sync>>;

// TODO: make the graph lives on another thread?
#[derive(Default)]
pub(crate) struct ReactiveGraph {
    pub(crate) storage: Storage,
    pub(crate) current: Option<AnySubscriber>,
}

unsafe impl Send for ReactiveGraph {}
unsafe impl Sync for ReactiveGraph {}

/*
#########################################################
#                                                       #
#                         Graph                         #
#                                                       #
#########################################################
*/

pub(crate) struct Graph;

impl Graph {
    pub(crate) fn insert<R: Reactive + Send + Sync + 'static>(r: R) -> Node<R> {
        let mut graph = GRAPH.get_or_init(Default::default).write().unwrap();
        let id = graph.storage.insert(Box::new(r));
        Node { id, marker: PhantomData }
    }

    pub(crate) fn with<U>(f: impl FnOnce(&ReactiveGraph) -> U) -> U {
        f(&GRAPH.get_or_init(Default::default).read().unwrap())
    }

    // pub(crate) fn with_mut<U>(f: impl FnOnce(&mut ReactiveGraph) -> U) -> U {
    //     f(&mut GRAPH.get_or_init(Default::default).write().unwrap())
    // }

    pub(crate) fn with_downcast<R, F, U>(node: &Node<R>, f: F) -> U
    where
        R: Reactive + Send + Sync + 'static,
        F: FnOnce(&R) -> U,
    {
        let graph = GRAPH.get_or_init(Default::default).read().unwrap();
        let r = graph
            .storage
            .get(&node.id)
            .and_then(|any| any.downcast_ref::<R>())
            .unwrap();
        f(&r)
    }

    pub(crate) fn try_with_downcast<R, F, U>(node: &Node<R>, f: F) -> Option<U>
    where
        R: Reactive + Send + Sync + 'static,
        F: FnOnce(Option<&R>) -> Option<U>,
    {
        let graph = GRAPH.get_or_init(Default::default).read().unwrap();
        graph
            .storage
            .get(&node.id)
            .and_then(|any| f(any.downcast_ref::<R>()))
    }

    pub(crate) fn set_scope(subscriber: Option<AnySubscriber>) -> Option<AnySubscriber> {
        let mut graph = GRAPH.get_or_init(Default::default).write().unwrap();
        let prev = graph.current.take();
        graph.current = subscriber;
        prev
    }

    pub(crate) fn remove<R: Reactive + Send + Sync>(node: &Node<R>) -> Option<Box<dyn Any + Send + Sync>> {
        let mut graph = GRAPH.get_or_init(Default::default).write().unwrap();
        graph.storage.remove(&node.id)
    }

    pub(crate) fn is_removed<R: Reactive + Send + Sync>(node: &Node<R>) -> bool {
        let graph = GRAPH.get_or_init(Default::default).read().unwrap();
        graph.storage.get(&node.id).is_none()
    }
}

/*
#########################################################
#                                                       #
#                          Node                         #
#                                                       #
#########################################################
*/

// type Map = RwLock<IndexMap<ReactiveId, Box<dyn Any + Send + Sync>>>;
// pub(crate) static STORAGE: OnceLock<Map> = OnceLock::new();

entity! {
    pub(crate) ReactiveId,
}

pub(crate) struct Node<R> {
    pub(crate) id: ReactiveId,
    marker: PhantomData<R>,
}

impl<R> Clone for Node<R> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            marker: PhantomData,
        }
    }
}

impl<R> PartialEq for Node<R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<R> PartialOrd for Node<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
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

#[cfg(test)]
mod signal_test {
    use crate::Signal;
    use crate::reactive_traits::*;

    #[test]
    fn signal() {
        let (counter, set_counter) = Signal::split(0i32);

        set_counter.update(|num| *num += 1);
        assert_eq!(counter.get(), 1);

        set_counter.set(-69);
        assert_eq!(counter.get(), -69);

        let r = counter.try_with(|num| num.map(ToString::to_string));
        assert!(r.is_some());
        assert_eq!(r.unwrap().parse(), Ok(-69));
    }

    #[test]
    fn derive() {
        let rw = Signal::new(0i32);
        let (counter, set_counter) = Signal::split(0i32);

        set_counter.set(69);
        rw.update(|num| *num = counter.get());
        assert_eq!(rw.get(), 69);
    }

    #[test]
    #[should_panic]
    fn dispose() {
        let (num, set_num) = Signal::split(0i32);
        let double = || num.get() * 2;

        set_num.set(1);
        assert_eq!(double(), 2);

        num.dispose();
        set_num.set(2);
        assert_eq!(double(), 2);
    }
}
