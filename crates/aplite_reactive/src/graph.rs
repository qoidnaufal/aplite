use std::cell::RefCell;

use aplite_storage::{IndexMap, Entity, entity};

use crate::stored_value::StoredValue;
use crate::subscriber::{Subscriber, AnySubscriber, WeakSubscriber};

thread_local! {
    pub(crate) static GRAPH: ReactiveGraph = ReactiveGraph::default();
}

entity! {
    pub ReactiveId,
    pub EffectId,
}

/*
#########################################################
#                                                       #
#                         Graph                         #
#                                                       #
#########################################################
*/

type ValueStorage = IndexMap<ReactiveId, StoredValue>;
type SubscriberStorage = IndexMap<EffectId, AnySubscriber>;

#[derive(Default)]
pub(crate) struct ReactiveGraph {
    pub(crate) storage: RefCell<ValueStorage>,
    pub(crate) current: RefCell<Option<WeakSubscriber>>,
    pub(crate) subscribers: RefCell<SubscriberStorage>,
}

impl ReactiveGraph {
    pub(crate) fn track(&self, id: &ReactiveId) {
        let current = self.current.borrow();

        if let Some(weak_subscriber) = current.as_ref()
        && let Some(value) = self.storage.borrow_mut().get_mut(id) {
            #[cfg(test)] eprintln!(" TRACKING: {id:?} inside {weak_subscriber:?}");
            weak_subscriber.add_source(*id);
            value.add_subscriber(weak_subscriber.clone());
        }
    }

    pub(crate) fn untrack(&self, id: &ReactiveId) {
        if let Some(value) = self.storage.borrow_mut().get_mut(id) {
            #[cfg(test)] eprintln!("UNTRACKED: {id:?}");
            value.clear_subscribers();
        }
    }

    pub(crate) fn notify_subscribers(&self, id: &ReactiveId) {
        if let Some(stored_value) = self.storage.borrow().get(id) {
            #[cfg(test)] eprintln!("\nNOTIFYING: {id:?} is notifying the subscribers");
            stored_value.notify_subscribers();
        }
    }

    pub(crate) fn swap_current(
        &self,
        subscriber: Option<WeakSubscriber>,
    ) -> Option<WeakSubscriber> {
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
