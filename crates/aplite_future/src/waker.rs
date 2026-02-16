use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Waker;

pub(crate) struct AtomicWaker {
    state: AtomicUsize,
    waker: Mutable<Option<Waker>>,
}

#[repr(transparent)]
struct Mutable<T> {
    value: T
}

impl<T: Sized> Mutable<T> {
    const fn new(value: T) -> Self {
        Self {
            value
        }
    }

    unsafe fn get(&self) -> *mut T {
        &self.value as *const T as *mut T
    }
}

unsafe impl Send for AtomicWaker {}
unsafe impl Sync for AtomicWaker {}

impl AtomicWaker {
    const EMPTY: usize = 0;
    const SET: usize = 0b01;
    const WAKING: usize = 0b10;

    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicUsize::new(Self::EMPTY),
            waker: Mutable::new(None),
        }
    }

    pub(crate) fn set(&self, new: &Waker) {
        let state = self.state
            .compare_exchange(
                Self::EMPTY,
                Self::SET,
                Ordering::Acquire,
                Ordering::Acquire
            )
            .unwrap_or_else(|current| current);

        match state {
            Self::EMPTY => {
                unsafe {
                    let opt = &mut *self.waker.get();

                    match &opt {
                        Some(old) if old.will_wake(new) => {},
                        _ => *opt = Some(new.clone()),
                    }

                    if self.state
                        .compare_exchange(
                            Self::SET,
                            Self::EMPTY,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_err()
                    {
                        let waker = (&mut *self.waker.get()).take().unwrap();
                        self.state.swap(Self::EMPTY, Ordering::AcqRel);
                        waker.wake();
                    }
                }
            },
            Self::WAKING => new.wake_by_ref(),
            _ => {}
        }
    }

    pub(crate) fn take(&self) -> Option<Waker> {
        match self.state.fetch_or(Self::WAKING, Ordering::AcqRel) {
            Self::EMPTY => {
                let waker = unsafe {
                    let opt = &mut *self.waker.get();
                    opt.take()
                };

                self.state.fetch_and(!Self::WAKING, Ordering::Release);

                waker
            }
            _ => None,
        }
    }
}
