use crate::runtime::RUNTIME;
use crate::traits::Subscriber;

pub struct Effect<R> {
    pub(crate) f: Box<dyn FnMut(Option<R>) -> R>,
}

impl<R> Effect<R> {
    pub fn new<F>(f: F)
    where
        F: FnMut(Option<R>) -> R + 'static,
        R: 'static,
    {
        RUNTIME.with(|rt| rt.create_effect(f))
    }

    pub(crate) fn run(&mut self) -> R {
        (self.f)(None)
    }
}

impl<R> Subscriber for Effect<R> {
    fn run(&mut self) {
        self.run();
    }
}

pub struct AnySubscriber(Box<dyn Subscriber>);

impl AnySubscriber {
    pub(crate) fn run(&mut self) {
        self.0.run();
    }
}

pub(crate) trait ToAnySubscriber {
    fn to_any_subscriber(self) -> AnySubscriber;
}

impl<T: Subscriber + 'static> ToAnySubscriber for T {
    fn to_any_subscriber(self) -> AnySubscriber {
        AnySubscriber(Box::new(self))
    }
}
