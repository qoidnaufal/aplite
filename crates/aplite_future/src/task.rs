use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Wake, Context, Poll};
use std::future::Future;
use std::sync::mpsc::Sender;

type PinnedFuture = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) struct Task {
    pub(crate) future: RwLock<Option<PinnedFuture>>,
    sender: Sender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let task = Arc::clone(&self);
        let _ = self.sender.send(task);
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub(crate) fn new<F>(sender: Sender<Arc<Task>>, future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self {
            future: RwLock::new(Some(Box::pin(future))),
            sender,
        }
    }
}

use std::time::Instant;
use std::time::Duration;

struct Sleep {
    start: Instant,
    duration: u64,
}

impl Sleep {
    #[inline(always)]
    fn new(millis: u64) -> Self {
        Self {
            start: Instant::now(),
            duration: millis,
        }
    }
}

pub async fn sleep(millis: u64) {
    Sleep::new(millis).await
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = self.start.elapsed();
        if now.as_secs() >= Duration::from_millis(self.duration).as_secs() {
            return Poll::Ready(());
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
