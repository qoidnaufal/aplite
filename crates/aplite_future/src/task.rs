use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Wake, Context, Poll};
use std::future::Future;

use crate::executor::SPAWNER;

type PinnedFuture = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) struct Task {
    pub(crate) future: RwLock<PinnedFuture>,
}

impl Task {
    pub(crate) fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self {
            future: RwLock::new(Box::pin(future)),
        }
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        if let Some(spawner) = SPAWNER.get() {
            let _ = spawner.send(self);
        }
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

use std::time::Instant;
use std::time::Duration;

struct Sleep {
    start: Instant,
    duration: Duration,
}

impl Sleep {
    #[inline(always)]
    fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }
}

pub async fn sleep(duration: Duration) {
    Sleep::new(duration).await
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = self.start.elapsed();
        if now.as_millis() >= self.duration.as_millis() {
            return Poll::Ready(());
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
