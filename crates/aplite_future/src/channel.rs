use std::sync::{Arc, Weak, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};
use std::pin::Pin;

use crate::stream::*;

pub struct Sender(Arc<Inner>);

pub struct Receiver(Weak<Inner>);

pub fn channel() -> (Sender, Receiver) {
    let inner = Arc::new(Inner::default());
    let rx = Receiver(Arc::downgrade(&inner));
    let tx = Sender(inner);
    (tx, rx)
}

#[derive(Default)]
struct Inner {
    state: AtomicBool,
    waker: RwLock<Option<Waker>>,
}

impl Sender {
    pub fn notify(&self) {
        if let Ok(mut guard) = self.0.waker.try_write()
        && let Some(waker) = guard.take()
        {
            self.0.state.store(true, Ordering::Relaxed);
            waker.wake();
        }
    }
}

impl Stream for Receiver {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(inner) = self.0.upgrade() {
            let mut waker = inner.waker.try_write().unwrap();
            *waker = Some(cx.waker().clone());

            if inner.state.swap(false, Ordering::Relaxed) {
                Poll::Ready(Some(()))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
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
            let (tx, mut rx) = channel();

            Executor::spawn_local(async move {
                while rx.next().await.is_some() {
                    eprintln!("notified")
                }
            });

            for _ in 0..3 {
                crate::runtime::sleep(1).await;
                tx.notify();
            }
        });

        runtime.run();
    }
}
