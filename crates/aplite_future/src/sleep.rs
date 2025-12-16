use std::pin::Pin;
use std::time::Instant;
use std::time::Duration;
use std::task::{Context, Poll};

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
