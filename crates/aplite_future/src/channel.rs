use std::sync::{Arc, Weak, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker, Wake};
use std::pin::Pin;

use crate::stream::*;

pub fn notifier() -> (Notifier, Rx) {
    let inner = Arc::new(NotifierState::new());
    let rx = Rx(Arc::downgrade(&inner));
    let tx = Notifier(inner);
    (tx, rx)
}

pub struct Notifier(Arc<NotifierState>);

pub struct Rx(Weak<NotifierState>);

struct NotifierState {
    dirty: AtomicBool,
    waker: AtomicWaker,
}

impl NotifierState {
    const fn new() -> Self {
        Self {
            dirty: AtomicBool::new(false),
            waker: AtomicWaker::new(),
        }
    }
}

impl Notifier {
    pub fn notify(&self) {
        self.0.dirty.store(true, Ordering::Relaxed);
        self.0.wake_by_ref();
    }
}

impl Clone for Notifier {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Wake for NotifierState {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl Stream for Rx {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.upgrade() {
            Some(inner) => {
                inner.waker.set(cx.waker());

                if inner.dirty.swap(false, Ordering::Relaxed) {
                    Poll::Ready(Some(()))
                } else {
                    Poll::Pending
                }
            },
            None => Poll::Ready(None),
        }
    }
}

impl Rx {
    pub fn recv(&mut self) -> impl Future<Output = Option<<Self as Stream>::Item>> {
        crate::stream::Recv {
            inner: Pin::new(self),
        }
    }
}

/*
#########################################################
#
# Mutable
#
#########################################################
*/

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

/*
#########################################################
#
# AtomicWaker
#
#########################################################
*/

struct AtomicWaker {
    state: AtomicUsize,
    waker: Mutable<Option<Waker>>,
}

unsafe impl Send for AtomicWaker {}
unsafe impl Sync for AtomicWaker {}

impl AtomicWaker {
    const EMPTY: usize = 0;
    const SET: usize = 0b01;
    const WAKING: usize = 0b10;

    const fn new() -> Self {
        Self {
            state: AtomicUsize::new(Self::EMPTY),
            waker: Mutable::new(None),
        }
    }

    fn set(&self, new: &Waker) {
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

    fn take(&self) -> Option<Waker> {
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

/*
#########################################################
#                                                       #
#                     Typed Channel                     #
#                                                       #
#########################################################
*/

pub fn async_channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(ChannelState::default());
    let rx = Receiver(Arc::downgrade(&inner));
    let tx = Sender(inner);
    (tx, rx)
}

pub struct Sender<T>(Arc<ChannelState<T>>);

pub struct Receiver<T>(Weak<ChannelState<T>>);

struct ChannelState<T> {
    value: RwLock<Option<T>>,
    waker: AtomicWaker,
}

impl<T> Default for ChannelState<T> {
    fn default() -> Self {
        Self {
            value: RwLock::new(None),
            waker: AtomicWaker::new(),
        }
    }
}

impl<T> Sender<T> {
    pub fn notify(&self, value: T) {
        *self.0.value.write().unwrap() = Some(value);
        self.0.wake_by_ref();
    }

    pub fn close(self) {
        drop(self)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>()
            && let Some(pending_data) = self.0.value.write().unwrap().take() {
            drop(pending_data)
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Wake for ChannelState<T> {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.upgrade() {
            Some(inner) => {
                inner.waker.set(cx.waker());

                if let Some(val) = inner.value.write().unwrap().take() {
                    Poll::Ready(Some(val))
                } else {
                    Poll::Pending
                }
            },
            None => Poll::Ready(None),
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> impl Future<Output = Option<<Self as Stream>::Item>> {
        crate::stream::Recv {
            inner: Pin::new(self),
        }
    }
}

/*
#########################################################
#                                                       #
#                         TEST                          #
#                                                       #
#########################################################
*/

#[cfg(test)]
mod channel_test {
    use std::time::Duration;
    use super::*;
    use crate::executor::Executor;
    use crate::sleep::sleep;

    #[test]
    fn poll() {
        let (tx, mut rx) = notifier();

        Executor::spawn(async move {
            while rx.recv().await.is_some() {
                eprintln!("notified")
            }
        });

        Executor::spawn(async move {
            for _ in 0..3 {
                tx.notify();
                sleep(Duration::from_secs(1)).await;
            }
        });

        #[derive(Debug)] struct Obj { _age: u8, _name: String }

        let (tx, mut rx) = async_channel::<Obj>();

        Executor::spawn(async move {
            while let Some(val) = rx.recv().await {
                eprintln!("received: {val:?}")
            }
        });

        Executor::spawn(async move {
            for i in 0..3 {
                tx.notify(Obj { _age: i, _name: i.to_string() });
                sleep(Duration::from_secs(1)).await;
            }
        });

        let now = std::time::Instant::now();
        loop {
            if now.elapsed() > std::time::Duration::from_secs(4) {
                break
            }
        }
    }
}
