use std::{cell::RefCell, ops::DerefMut, rc::Rc};

#[derive(Clone)]
pub struct Signal<T> {
    read: SignalRead<T>,
    write: SignalWrite<T>,
}

impl<T: Clone> Signal<T> {
    pub fn new(value: T) -> Self {
        let v = Rc::new(RefCell::new(value));
        Self {
            read: SignalRead(v.clone()),
            write: SignalWrite(v),
        }
    }

    pub fn get(&self) -> T {
        self.read.get()
    }

    pub fn set<F: FnOnce(&mut T) + 'static>(&self, f: F) {
        self.write.set(f);
    }
}

#[derive(Clone)]
pub struct SignalRead<T>(Rc<RefCell<T>>);

impl<T: Clone> SignalRead<T> {
    pub fn get(&self) -> T {
        let val = self.0.as_ref().borrow();
        val.clone()
    }
}

#[derive(Clone)]
pub struct SignalWrite<T>(Rc<RefCell<T>>);

impl<T: Clone> SignalWrite<T> {
    pub fn set<F: FnOnce(&mut T) + 'static>(&self, f: F) {
        let mut val = self.0.borrow_mut();
        let v = val.deref_mut();
        f(v)
    }
}

pub struct SignalRuntime {}
