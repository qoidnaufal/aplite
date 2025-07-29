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
    signal: AtomicBool,
    waker: RwLock<Option<Waker>>,
}

pub struct Channel;

impl Channel {
    pub fn new() -> (Sender, Receiver) {
        let inner = Arc::new(Inner::default());
        let rx = Receiver(Arc::downgrade(&inner));
        let tx = Sender(inner);
        (tx, rx)
    }
}

impl Sender {
    pub fn notify(&self) {
        #[cfg(test)] eprintln!(">> notifying");
        self.0.signal.store(true, Ordering::Relaxed);
        self.0.wake_by_ref();
    }
}

impl Inner {
    fn set_waker(&self, new: &Waker) {
        let mut inner = self.waker.try_write().unwrap();
        match inner.as_ref() {
            Some(old) if old.will_wake(new) => {},
            _ => *inner = {
                // eprintln!(">> storing waker");
                Some(new.clone())
            },
        }
    }
}

impl Wake for Inner {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let mut lock = self.waker.try_write().unwrap();
        if let Some(waker) = lock.take() {
            // eprintln!(">> waking up");
            waker.wake();
        }
    }
}

impl Stream for Receiver {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(inner) = self.0.upgrade() {
            // eprintln!(">> polled");
            inner.set_waker(cx.waker());

            if inner.signal.swap(false, Ordering::Relaxed) {
                // eprintln!("\n   +++++ READYSOME +++++\n");
                Poll::Ready(Some(()))
            } else {
                // eprintln!("\n   +++++  PENDING  +++++\n");
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
}

impl Receiver {
    pub async fn recv(&mut self) -> Option<()> {
        PollReceive { inner: self }.await
    }
}

struct PollReceive<'a, T>
where
    T: ?Sized + Stream
{
    inner: &'a mut T
}

impl<T> Future for PollReceive<'_, T>
where
    T: ?Sized + Stream + Unpin,
{
    type Output = Option<T::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

#[cfg(test)]
mod channel_test {
    use super::*;
    use crate::{Runtime, Executor};

    #[test]
    fn poll() {
        let runtime = Runtime::init_local();

        runtime.spawn_local(async {
            let (tx, mut rx) = Channel::new();

            Executor::spawn_local(async move {
                while rx.recv().await.is_some() {
                    eprintln!("notified")
                }
            });

            for _ in 0..3 {
                crate::task::sleep(1).await;
                tx.notify();
            }
        });

        runtime.run();
    }
}
