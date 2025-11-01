use std::sync::{Arc, Weak, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use std::pin::Pin;
use std::task::Wake;

use crate::stream::*;

#[derive(Clone)]
pub struct Sender(Arc<Inner>);

pub struct Receiver(Weak<Inner>);

#[derive(Default)]
struct Inner {
    dirty: AtomicBool,
    waker: RwLock<Option<Waker>>,
}

pub struct Channel;

impl Channel {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> (Sender, Receiver) {
        let inner = Arc::new(Inner::default());
        let rx = Receiver(Arc::downgrade(&inner));
        let tx = Sender(inner);
        (tx, rx)
    }
}

impl Sender {
    pub fn notify(&self) {
        self.0.dirty.store(true, Ordering::Relaxed);
        self.0.wake_by_ref();
    }

    pub fn close(&self) {
        unsafe {
            let strong_count = Arc::strong_count(&self.0);
            for _ in 0..strong_count {
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

impl Receiver {
    pub async fn recv(&mut self) -> Option<()> {
        self.await
    }
}

impl Future for Receiver {
    type Output = Option<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.poll_next(cx)
    }
}

#[cfg(test)]
mod channel_test {
    use super::*;
    use crate::Executor;

    #[test]
    fn poll() {
        Executor::init(1);

        Executor::spawn(async {
            let (tx, mut rx) = Channel::new();

            Executor::spawn(async move {
                while rx.recv().await.is_some() {
                    eprintln!("notified")
                }
            });

            for _ in 0..3 {
                crate::task::sleep(1).await;
                tx.notify();
            }
        });
    }
}
