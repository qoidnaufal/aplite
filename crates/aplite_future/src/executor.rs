use std::sync::{Arc, OnceLock};
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
use std::task::{Waker, Context, Poll};
use std::thread;

use crate::task::Task;

/*
#########################################################
#
# Spawner & Worker
#
#########################################################
*/

pub(crate) static SPAWNER: OnceLock<Spawner> = const { OnceLock::new() };

#[derive(Debug)]
pub(crate) struct Spawner(SyncSender<Arc<Task>>);

impl Spawner {
    pub(crate) fn send(&self, task: Arc<Task>) {
        self.0.send(task).unwrap()
    }
}

unsafe impl Send for Spawner {}
unsafe impl Sync for Spawner {}

struct Worker {
    rx: Receiver<Arc<Task>>,
}

impl Worker {
    fn work(&self) {
        while let Ok(task) = self.rx.recv() {
            if let Ok(mut future) = task.future.write() {
                let waker = Waker::from(Arc::clone(&task));
                let cx = &mut Context::from_waker(&waker);

                match future.as_mut().poll(cx) {
                    Poll::Ready(_) => drop(future),
                    Poll::Pending => continue,
                }
            }

            drop(task)
        };
    }
}

unsafe impl Send for Worker {}
unsafe impl Sync for Worker {}

pub struct Executor;

impl Executor {
    pub fn spawn(future: impl Future<Output = ()> + 'static) {
        let spawner = SPAWNER.get_or_init(|| {
            let (tx, rx) = sync_channel(128);
            let worker = Worker { rx };

            let worker_thread = thread::Builder::new().name("async worker".to_string());
            let _ = worker_thread.spawn(move || worker.work());

            Spawner(tx)
        });

        let task = Arc::new(Task::new(future));
        spawner.send(task);
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
        Executor::spawn(async {
            let result = dummy_async().await;
            println!("{:?}", result);
            assert!(result.is_ok());
        });

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
