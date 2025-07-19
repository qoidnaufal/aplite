use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::task::{Context, Wake, Waker};

pub fn spawn<T: 'static>(future: impl Future<Output = T> + 'static + Send) -> T {
    let (executor, spawner) = create_executor();

    spawner.spawn(future);
    drop(spawner);

    executor.run()
}

fn create_executor<T: 'static>() -> (Executor<T>, Spawner<T>) {
    const MAX_TASK: usize = 1024;
    let (tx, rx) = sync_channel(MAX_TASK);
    (Executor { rx }, Spawner { tx })
}

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

struct Task<T> {
    future: RwLock<Option<PinnedFuture<T>>>,
    sender: SyncSender<Arc<Task<T>>>,
}

impl<T: 'static> Wake for Task<T> {
    fn wake(self: Arc<Self>) {
        let cloned = Arc::clone(&self);
        self.sender
            .try_send(cloned)
            .expect("Buffer has enough room")
    }
}

unsafe impl<T> Send for Task<T> {}
unsafe impl<T> Sync for Task<T> {}

#[derive(Clone)]
struct Spawner<T> {
    tx: SyncSender<Arc<Task<T>>>,
}

impl<T> Spawner<T> {
    fn spawn(&self, future: impl Future<Output = T> + 'static + Send) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: RwLock::new(Some(future)),
            sender: self.tx.clone(),
        });
        self.tx.try_send(task).expect("Buffer has enough room")
    }
}

struct Executor<T> {
    rx: Receiver<Arc<Task<T>>>
}

impl<T: 'static> Executor<T> {
    fn run(&self) -> T {
        let (tx, rx) = sync_channel::<T>(1);
        while let Ok(task) = self.rx.try_recv() {
            let mut future_slot = task.future.write().unwrap();

            if let Some(mut future) = future_slot.take() {
                let waker = Waker::from(Arc::clone(&task));
                let cx = &mut Context::from_waker(&waker);

                match future.as_mut().poll(cx) {
                    std::task::Poll::Ready(value) => tx.try_send(value).unwrap(),
                    std::task::Poll::Pending => *future_slot = Some(future),
                }
            }
        };
        rx.try_recv().unwrap()
    }
}

#[cfg(test)]
mod executor_test {
    use super::spawn;

    async fn dummy_async() -> std::io::Result<String> {
        use std::fs::File;
        use std::io::Read;

        let mut buf = String::new();
        let mut file = File::open("src/poll.rs")?;
        file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    #[test]
    fn spawn_test() {
        let result = spawn(async move {
            dummy_async().await
        });

        eprintln!("{result:?}");
        assert!(result.is_ok())
    }
}
