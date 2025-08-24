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
        #[cfg(test)] eprintln!(">> notifying");
        self.0.signal.store(true, Ordering::Relaxed);
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
            _ => *inner = {
                #[cfg(test)] eprintln!(">> storing waker");
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
        if let Ok(mut lock) = self.waker.write()
            && let Some(waker) = lock.take()
        {
            #[cfg(test)] eprintln!(">> waking up");
            waker.wake();
        }
    }
}

impl Stream for Receiver {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(inner) = self.0.upgrade() {
            #[cfg(test)] eprintln!(">> polled");
            inner.set_waker(cx.waker());

            if inner.signal.swap(false, Ordering::Relaxed) {
                #[cfg(test)] eprintln!("\n   +++++ READYSOME +++++\n");
                Poll::Ready(Some(()))
            } else {
                #[cfg(test)] eprintln!("\n   +++++  PENDING  +++++\n");
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
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

// struct Recv<'a, T>
// where
//     T: ?Sized + Stream
// {
//     inner: &'a mut T
// }

// impl<T> Future for Recv<'_, T>
// where
//     T: ?Sized + Stream + Unpin,
// {
//     type Output = Option<T::Item>;

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         Pin::new(&mut self.inner).poll_next(cx)
//     }
// }

#[cfg(test)]
mod channel_test {
    use super::*;
    use crate::Executor;

    #[test]
    fn poll() {
        Executor::init();

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
