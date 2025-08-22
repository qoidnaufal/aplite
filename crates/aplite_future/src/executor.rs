use std::sync::{Arc, OnceLock};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::task::{Waker, Context, Poll};
use std::thread;

use crate::task::Task;

pub(crate) static SPAWNER: OnceLock<Sender<Arc<Task>>> = OnceLock::new();

struct Worker {
    rx: Receiver<Arc<Task>>,
}

impl Worker {
    pub fn work(&self) {
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
    pub fn init() {
        let (tx, rx) = channel();
        let worker = Worker { rx };

        SPAWNER.set(tx).expect("Executor can only be initiated once");

        let worker_thread = thread::Builder::new().name("async worker".to_string());
        worker_thread.spawn(move || worker.work()).unwrap();
    }

    pub fn spawn(future: impl Future<Output = ()> + 'static) {
        let spawner = SPAWNER.get().expect("Executor should has been initialized once");
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
        Executor::init();

        Executor::spawn(async {
            let result = dummy_async().await;
            assert!(result.is_ok());
        });
    }
}
