use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;
use std::any::Any;

use aplite_storage::{IndexMap, Entity, entity};

use crate::subscriber::AnySubscriber;
use crate::reactive_traits::*;

thread_local! {
    pub(crate) static GRAPH: ReactiveGraph = ReactiveGraph::default();
}

entity! {
    pub(crate) ReactiveId,
}

pub(crate) struct ReactiveNode<R> {
    pub(crate) id: ReactiveId,
    marker: PhantomData<R>,
}

impl<R> Clone for ReactiveNode<R> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            marker: PhantomData,
        }
    }
}

impl<R> PartialEq for ReactiveNode<R> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<R> PartialOrd for ReactiveNode<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<R> Ord for ReactiveNode<R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<R> Copy for ReactiveNode<R> {}
impl<R> Eq for ReactiveNode<R> {}

impl<R> std::hash::Hash for ReactiveNode<R> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<R> std::fmt::Debug for ReactiveNode<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<R>())
            .field("id", &self.id)
            .finish()
    }
}

/*
#########################################################
#                                                       #
#                         Graph                         #
#                                                       #
#########################################################
*/

type Storage = IndexMap<ReactiveId, Arc<dyn Any>>;

#[derive(Default)]
pub(crate) struct ReactiveGraph {
    pub(crate) storage: RefCell<Storage>,
    pub(crate) current: RefCell<Option<AnySubscriber>>,
}

impl ReactiveGraph {
    pub(crate) fn insert<R: Reactive + 'static>(&self, r: R) -> ReactiveNode<R> {
        let id = self.storage.borrow_mut().insert(Arc::new(r));
        ReactiveNode { id, marker: PhantomData }
    }

    pub(crate) fn get<R: Reactive>(&self, node: &ReactiveNode<R>) -> Option<Arc<dyn Any>> {
        self.storage.borrow().get(&node.id).map(|arc| Arc::clone(&arc))
    }

    pub(crate) fn remove<R: Reactive>(&self, node: &ReactiveNode<R>) -> Option<Arc<dyn Any>> {
        self.storage.borrow_mut().remove(&node.id)
    }

    pub(crate) fn swap_current(
        &self,
        subscriber: Option<AnySubscriber>,
    ) -> Option<AnySubscriber> {
        self.current.replace(subscriber)
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

        let r = counter.read(|num| num.to_string());
        assert_eq!(r.parse(), Ok(-69));
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
