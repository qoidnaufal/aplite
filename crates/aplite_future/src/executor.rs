use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::task::{Context, Wake, Waker};

pub fn spawn(future: impl Future<Output = ()> + 'static + Send) {
    let (executor, spawner) = create_executor();

    spawner.spawn(future);
    drop(spawner);

    executor.run();
}

fn create_executor() -> (Executor, Spawner) {
    const MAX_TASK: usize = 1024;
    let (tx, rx) = sync_channel(MAX_TASK);
    (Executor { rx }, Spawner { tx })
}

type BoxedFuture = Pin<Box<dyn Future<Output = ()>>>;

struct Task {
    future: RwLock<Option<BoxedFuture>>,
    sender: SyncSender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let cloned = Arc::clone(&self);
        self.sender
            .try_send(cloned)
            .expect("Buffer has enough room")
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

#[derive(Clone)]
struct Spawner {
    tx: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: RwLock::new(Some(future)),
            sender: self.tx.clone(),
        });
        self.tx.try_send(task).expect("Buffer has enough room")
    }
}

struct Executor {
    rx: Receiver<Arc<Task>>
}

impl Executor {
    fn run(&self) {
        while let Ok(task) = self.rx.try_recv() {
            let mut future_slot = task.future.write().unwrap();

            if let Some(mut future) = future_slot.take() {
                let waker = Waker::from(Arc::clone(&task));
                let cx = &mut Context::from_waker(&waker);

                if future.as_mut().poll(cx).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

#[cfg(test)]
mod executor_test {
    use super::spawn;

    async fn dummy_async() -> std::io::Result<String> {
        use std::fs::File;
        use std::io::Read;

        let mut buf = String::new();
        let mut file = File::open("src/executor.rs")?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    #[test]
    fn spawn_test() {
        let (tx, rx) = std::sync::mpsc::channel();

        spawn(async move {
            let result = dummy_async().await;
            if let Ok(result) = result {
                tx.send(result).unwrap();
                drop(tx);
            };
        });

        eprintln!("outside");

        spawn(async move {
            if let Ok(val) = rx.recv() {
                assert!(val.len() > 0);
                eprintln!("{}", val.len());
            }
        });
    }
}
