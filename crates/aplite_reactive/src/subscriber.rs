use std::rc::Rc;

pub(crate) struct AnySubscriber(Rc<dyn Subscriber>);

// pub(crate) struct WeakSubscriber(Weak<dyn Subscriber>);

// impl AnySubscriber {
//     pub(crate) fn downgrade(&self) -> WeakSubscriber {
//         WeakSubscriber(Rc::downgrade(&self.0))
//     }
// }

// impl WeakSubscriber {
//     pub(crate) fn upgrade(&self) -> Option<AnySubscriber> {
//         self.0
//             .upgrade()
//             .map(|s| AnySubscriber(s))
//     }
// }

impl Clone for AnySubscriber {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl PartialEq for AnySubscriber {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for AnySubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnySubscriber")
            .field("address", &Rc::as_ptr(&self.0).addr())
            .finish()
    }
}

// impl Clone for WeakSubscriber {
//     fn clone(&self) -> Self {
//         Self(Weak::clone(&self.0))
//     }
// }

// impl PartialEq for WeakSubscriber {
//     fn eq(&self, other: &Self) -> bool {
//         Weak::ptr_eq(&self.0, &other.0)
//     }
// }

// impl std::fmt::Debug for WeakSubscriber {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Subscriber")
//             .field("id", &self.0.as_ptr().addr())
//             .finish()
//     }
// }

pub(crate) trait Subscriber {
    fn notify(&self);
}

impl Subscriber for AnySubscriber {
    fn notify(&self) {
        self.0.notify();
    }
}

// impl Subscriber for WeakSubscriber {
//     fn notify(&self) {
//         if let Some(any_subscriber) = self.upgrade() {
//             any_subscriber.notify();
//         }
//     }
// }

pub(crate) trait ToAnySubscriber {
    fn to_any_subscriber(self) -> AnySubscriber;
}

impl<T: Subscriber + 'static> ToAnySubscriber for T {
    fn to_any_subscriber(self) -> AnySubscriber {
        AnySubscriber(Rc::new(self))
    }
}
