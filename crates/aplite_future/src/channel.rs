use std::sync::{Arc, Weak, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use std::pin::Pin;
use std::task::Wake;

use crate::stream::*;

pub fn aplite_channel() -> (Sender, Receiver) {
    let inner = Arc::new(Inner::default());
    let rx = Receiver(Arc::downgrade(&inner));
    let tx = Sender(inner);
    (tx, rx)
}

#[derive(Clone)]
pub struct Sender(Arc<Inner>);

pub struct Receiver(Weak<Inner>);

#[derive(Default)]
struct Inner {
    dirty: AtomicBool,
    waker: RwLock<Option<Waker>>,
}

impl Sender {
    pub fn notify(&self) {
        self.0.dirty.store(true, Ordering::Relaxed);
        self.0.wake_by_ref();
    }

    pub fn close(&self) {
        unsafe {
            for _ in 0..Arc::strong_count(&self.0) {
                Arc::decrement_strong_count(Arc::as_ptr(&self.0))
            }
        }
    }
}

impl Inner {
    fn set_waker(&self, new: &Waker) {
        let mut inner = self.waker.write().unwrap();
        match inner.as_ref() {
            Some(old) if old.will_wake(new) => {},
            _ => *inner = Some(new.clone()),
        }
    }
}

impl Wake for Inner {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Ok(mut lock) = self.waker.write()
            && let Some(waker) = lock.take()
        {
            waker.wake();
        }
    }
}

impl Stream for Receiver {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.upgrade() {
            Some(inner) => {
                inner.set_waker(cx.waker());

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

impl Future for Receiver {
    type Output = Option<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_next(cx)
    }
}

impl Receiver {
    pub async fn recv(&mut self) -> Option<()> {
        self.await
    }
}

/*
#########################################################
#                                                       #
#                     Typed Channel                     #
#                                                       #
#########################################################
*/

pub fn aplite_typed_channel<T>() -> (TypedSender<T>, TypedReceiver<T>) {
    let inner = Arc::new(TypedInner::default());
    let rx = TypedReceiver(Arc::downgrade(&inner));
    let tx = TypedSender(inner);
    (tx, rx)
}

#[derive(Clone)]
pub struct TypedSender<T>(Arc<TypedInner<T>>);

pub struct TypedReceiver<T>(Weak<TypedInner<T>>);

struct TypedInner<T> {
    value: RwLock<Option<T>>,
    waker: RwLock<Option<Waker>>,
}

impl<T> Default for TypedInner<T> {
    fn default() -> Self {
        Self {
            value: RwLock::new(None),
            waker: RwLock::new(None),
        }
    }
}

impl<T> TypedSender<T> {
    pub fn notify(&self, value: T) {
        *self.0.value.write().unwrap() = Some(value);
        self.0.wake_by_ref();
    }

    pub fn close(&self) {
        unsafe {
            for _ in 0..Arc::strong_count(&self.0) {
                Arc::decrement_strong_count(Arc::as_ptr(&self.0))
            }
        }
    }
}

impl<T> TypedInner<T> {
    fn set_waker(&self, new: &Waker) {
        let mut inner = self.waker.write().unwrap();
        match inner.as_ref() {
            Some(old) if old.will_wake(new) => {},
            _ => *inner = Some(new.clone()),
        }
    }
}

impl<T> Wake for TypedInner<T> {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if let Ok(mut lock) = self.waker.write()
            && let Some(waker) = lock.take()
        {
            waker.wake();
        }
    }
}

impl<T> Stream for TypedReceiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.upgrade() {
            Some(inner) => {
                inner.set_waker(cx.waker());

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

impl<T> Future for TypedReceiver<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_next(cx)
    }
}

impl<T> TypedReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.await
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
    use crate::task::sleep;

    #[test]
    fn poll() {
        Executor::init(4);

        let (tx, mut rx) = aplite_channel();

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

        let (tx, mut rx) = aplite_typed_channel::<Obj>();

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
