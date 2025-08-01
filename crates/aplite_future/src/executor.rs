use std::sync::{Arc, Weak, OnceLock, RwLock};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::task::{Waker, Context};

use crate::task::Task;

thread_local! {
    pub(crate) static CURRENT: OnceLock<WeakSender> = OnceLock::new();
}

#[derive(Debug)]
pub(crate) struct WeakSender(Weak<Sender<Arc<Task>>>);

type ArcSender = Arc<Sender<Arc<Task>>>;

pub struct Runtime {
    tx: ArcSender,
    rx: Receiver<Arc<Task>>,
}

impl Runtime {
    pub fn init() -> Self {
        let (tx, rx) = channel();
        let this = Self { tx: Arc::new(tx), rx };

        CURRENT.with(|cell| {
            cell.set(WeakSender::new(&this.tx))
                .expect("There should be no other runtime");
        });

        this
    }

    pub fn spawn_local(&self, future: impl Future<Output = ()> + 'static) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: RwLock::new(Some(future)),
            sender: WeakSender::new(&self.tx),
        });
        let _ = self.tx.send(task);
    }

    pub fn run(&self) {
        while let Ok(task) = self.rx.try_recv() {
            if let Ok(mut lock) = task.future.try_write()
            && let Some(mut future) = lock.take()
            {
                let waker = Waker::from(Arc::clone(&task));
                let cx = &mut Context::from_waker(&waker);

                match future.as_mut().poll(cx) {
                    std::task::Poll::Ready(()) => drop(future),
                    std::task::Poll::Pending => *lock = Some(future),
                }
            }
        };
    }
}

impl WeakSender {
    pub(crate) fn new(tx: &ArcSender) -> Self {
        Self(Arc::downgrade(tx))
    }

    #[inline(always)]
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

pub struct Executor;

impl Executor {
    pub fn spawn_local(future: impl Future<Output = ()> + 'static) {
        CURRENT.with(|cell| {
            if let Some(spawner) = cell.get() {
                let task = Arc::new(Task {
                    future: RwLock::new(Some(Box::pin(future))),
                    sender: spawner.clone(),
                });
                spawner.send(task);
            }
        });
    }
}

#[cfg(test)]
mod runtime_test {
    use super::Executor;
    use crate::executor::Runtime;
    
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
        let runtime = Runtime::init();

        runtime.spawn_local(async {
            Executor::spawn_local(async {
                let result = dummy_async().await;
                assert!(result.is_ok());
            });
        });

        runtime.run();
    }
}
