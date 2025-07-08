use std::rc::Rc;

pub(crate) trait Subscriber {
    fn run(&self);
}

pub(crate) struct AnySubscriber(Rc<dyn Subscriber>);

impl Clone for AnySubscriber {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl AnySubscriber {
    pub(crate) fn run(&self) {
        self.0.run();
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
