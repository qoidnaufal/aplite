use std::pin::Pin;
use std::sync::{Arc, Weak, OnceLock, RwLock};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::task::{Waker, Context};

use crate::task::Task;

thread_local! {
    pub(crate) static CURRENT_RUNTIME: OnceLock<WeakSender> = OnceLock::new();
}

#[derive(Debug)]
pub(crate) struct WeakSender(Weak<Sender<Arc<Task>>>);

impl WeakSender {
    pub(crate) fn new(tx: &ArcSender) -> Self {
        Self(Arc::downgrade(tx))
    }

    pub(crate) fn upgrade(&self) -> Option<ArcSender> {
        self.0.upgrade()
    }

    pub(crate) fn send(&self, task: Arc<Task>) {
        if let Some(sender) = self.upgrade() {
            let _ = sender.send(task);
        }
    }
}

impl Clone for WeakSender {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

type ArcSender = Arc<Sender<Arc<Task>>>;

pub struct Runtime {
    _tx: ArcSender,
    rx: Receiver<Arc<Task>>,
}

impl Runtime {
    pub fn init_local() -> Self {
        let (tx, rx) = channel();
        let tx = Arc::new(tx);
        CURRENT_RUNTIME.with(|cell| {
            cell.set(WeakSender::new(&tx)).expect("There should be no other runtime");
        });
        Self { _tx: tx, rx }
    }

    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'static + Send) {
        CURRENT_RUNTIME.with(|cell| {
            if let Some(spawner) = cell.get() {
                let future = Box::pin(future);
                let task = Arc::new(Task {
                    future: RwLock::new(Some(future)),
                    sender: spawner.clone(),
                });
                if let Some(spawner) = spawner.upgrade() {
                    let _ = spawner.send(task);
                }
            }
        });
    }

    pub fn run(&self) {
        while let Ok(task) = self.rx.try_recv() {
            let mut future_slot = task.future.write().unwrap();

            if let Some(mut future) = future_slot.take() {
                let waker = Waker::from(Arc::clone(&task));
                let cx = &mut Context::from_waker(&waker);

                match future.as_mut().poll(cx) {
                    std::task::Poll::Ready(()) => {},
                    std::task::Poll::Pending => *future_slot = Some(future),
                }
            }
        };
    }
}

struct Sleep {
    start: std::time::Instant,
    duration: std::time::Duration,
}

impl Sleep {
    #[inline(always)]
    fn new(duration: std::time::Duration) -> Self {
        Self {
            start: std::time::Instant::now(),
            duration,
        }
    }
}

pub async fn sleep(secs: u64) {
    Sleep::new(std::time::Duration::from_secs(secs)).await
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
