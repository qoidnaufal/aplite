use std::sync::{Arc, RwLock};
use std::pin::Pin;
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::task::{Wake, Waker, Context, Poll};
use std::thread;

/*
#########################################################
#
# Task
#
#########################################################
*/

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
        Spawner::send(self);
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

/*
#########################################################
#
# Spawner & Worker
#
#########################################################
*/

static SPAWNER: RwLock<Spawner> = RwLock::new(Spawner::new());

struct Spawner(Option<SyncSender<Arc<Task>>>);

impl Spawner {
    const fn new() -> Self {
        Self(None)
    }

    fn init(tx: SyncSender<Arc<Task>>) {
        if SPAWNER.read().unwrap().0.is_some() {
            drop(tx);
            return;
        }
        SPAWNER.write().unwrap().0 = Some(tx);
    }

    fn send(task: Arc<Task>) {
        if let Some(spawner) = SPAWNER.read().unwrap().0.as_ref() {
            let _ = spawner.send(task);
        }
    }
}

unsafe impl Send for Spawner {}
unsafe impl Sync for Spawner {}

struct Worker {
    rx: Receiver<Arc<Task>>,
}

impl Worker {
    fn work(&self) {
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

            Spawner::init(tx);

            let worker_thread = thread::Builder::new().name("async worker".to_string());
            let _ = worker_thread.spawn(move || worker.work());
        }
    }

    pub fn spawn(future: impl Future<Output = ()> + 'static) {
        let task = Arc::new(Task::new(future));
        Spawner::send(task);
    }

    pub fn deinit() {
        let sender = SPAWNER.write().unwrap().0.take();
        drop(sender);
    }
}

#[cfg(test)]
mod executor_test {
    use super::*;
    
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
            println!("{:?}", result);
            assert!(result.is_ok());
        });

        std::thread::sleep(std::time::Duration::from_secs(1));

        Executor::deinit();
    }
}
