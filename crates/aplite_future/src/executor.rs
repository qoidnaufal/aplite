use std::sync::{Arc, OnceLock, RwLock};
use std::pin::Pin;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::task::{Wake, Waker, Context, Poll};
use std::thread;

pub(crate) static SPAWNER: OnceLock<SyncSender<Arc<Task>>> = OnceLock::new();

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

struct Worker {
    rx: Receiver<Arc<Task>>,
}

impl Worker {
    pub fn work(&self) {
        // WARN: is this a busy loop?
        while let Ok(task) = self.rx.recv() {
            if let Ok(mut future) = task.future.write() {
                let waker = Waker::from(Arc::clone(&task));
                let cx = &mut Context::from_waker(&waker);

                match future.as_mut().poll(cx) {
                    Poll::Ready(_) => drop(future),
                    Poll::Pending => continue,
                }
            }

            // not sure if these drops are needed, just in case
            drop(task)
        };
    }
}

unsafe impl Send for Worker {}
unsafe impl Sync for Worker {}

pub struct Executor;

impl Executor {
    pub fn init(capacity: usize) {
        if capacity > 0 {
            let (tx, rx) = sync_channel(capacity);
            let worker = Worker { rx };

            SPAWNER.set(tx).expect("Executor can only be initialized once");

            let worker_thread = thread::Builder::new().name("async worker".to_string());
            let _ = worker_thread.spawn(move || worker.work());
        }
    }

    pub fn spawn(future: impl Future<Output = ()> + 'static) {
        let spawner = SPAWNER.get().expect("Executor must be initialized once");

        let task = Arc::new(Task::new(future));

        let _ = spawner.send(task);
    }
}

#[cfg(test)]
mod excutor_test {
    use super::Executor;
    
    async fn dummy_async() -> std::io::Result<String> {
        use std::fs::File;
        use std::io::Read;

        let mut buf = String::new();
        let mut file = File::open("src/lib.rs")?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    #[test]
    fn spawn_test() {
        Executor::init(1);

        Executor::spawn(async {
            let result = dummy_async().await;
            assert!(result.is_ok());
        });
    }
}
