use std::rc::Rc;

pub(crate) trait Subscriber {
    fn notify(&self);
}

pub(crate) struct AnySubscriber(Rc<dyn Subscriber>);

impl Clone for AnySubscriber {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl AnySubscriber {
    pub(crate) fn notify(&self) {
        self.0.notify();
    }
}

pub(crate) trait ToAnySubscriber {
    fn to_any_subscriber(self) -> AnySubscriber;
}

impl<T: Subscriber + 'static> ToAnySubscriber for T {
    fn to_any_subscriber(self) -> AnySubscriber {
        AnySubscriber(Rc::new(self))
    }
}
