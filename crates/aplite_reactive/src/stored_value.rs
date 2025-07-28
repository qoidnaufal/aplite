use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::graph::EffectId;

pub(crate) struct StoredValue {
    pub(crate) value: Rc<dyn Any>,
    pub(crate) subscribers: RefCell<Vec<EffectId>>,
}

impl StoredValue {
    pub(crate) fn new<T: Any + 'static>(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            subscribers: Default::default(),
        }
    }

    pub(crate) fn add_subscriber(&self, subscriber: EffectId) {
        let mut subscribers = self.subscribers.borrow_mut();
        if !subscribers.contains(&subscriber) {
            subscribers.push(subscriber);
        }
    }

    #[inline(always)]
    pub(crate) fn get_subscribers(&self) -> Vec<EffectId>  {
        self.subscribers.borrow().clone()
    }

    #[inline(always)]
    pub(crate) fn clear_subscribers(&self) {
        self.subscribers.borrow_mut().clear();
    }

    #[inline(always)]
    pub(crate) fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }
}
