use crate::runtime::RUNTIME;

pub struct Effect {
    pub(crate) f: Box<dyn Fn()>,
}

impl Effect {
    pub fn new<F: Fn() + 'static>(f: F) {
        RUNTIME.with(|rt| rt.create_effect(f))
    }

    pub(crate) fn run(&self) {
        (self.f)();
    }
}
