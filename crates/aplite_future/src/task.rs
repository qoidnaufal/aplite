use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Wake, Context};

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

struct Sleep {
    start: std::time::Instant,
    duration: std::time::Duration,
}

impl Sleep {
    #[inline(always)]
    fn new(millis: u64) -> Self {
        Self {
            start: std::time::Instant::now(),
            duration: std::time::Duration::from_millis(millis),
        }
    }
}

pub async fn sleep(millis: u64) {
    Sleep::new(millis).await
}

impl std::future::Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let now = self.start.elapsed();
        if now.as_secs() >= self.duration.as_secs() {
            return std::task::Poll::Ready(());
        }

        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}
