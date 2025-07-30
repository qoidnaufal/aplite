use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Wake, Context, Poll};
use std::future::Future;

use crate::runtime::WeakSender;

type PinnedFuture = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) struct Task {
    pub(crate) future: RwLock<Option<PinnedFuture>>,
    pub(crate) sender: WeakSender,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let cloned = Arc::clone(&self);
        if let Some(sender) = self.sender.upgrade() {
            let _ = sender.send(cloned);
        }
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

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
