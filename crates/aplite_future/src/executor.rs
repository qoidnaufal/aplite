use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use std::task::Wake;

use crate::runtime::CURRENT_RUNTIME;

// use aplite_storage::{Storage, entity, Entity};

pub(crate) type PinnedFuture = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) struct Task {
    pub(crate) future: RwLock<Option<PinnedFuture>>,
    pub(crate) sender: Sender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let cloned = Arc::clone(&self);
        let _ = self.sender.send(cloned);
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

pub struct Executor;

impl Executor {
    pub fn spawn_local(future: impl Future<Output = ()> + 'static + Send) {
        CURRENT_RUNTIME.with(|cell| {
            if let Some(spawner) = cell.get() {
                let future = Box::pin(future);
                let task = Arc::new(Task {
                    future: RwLock::new(Some(future)),
                    sender: spawner.clone(),
                });
                let _ = spawner.send(task);
            }
        });
    }
}

#[cfg(test)]
mod executor_test {
    use super::Executor;
    use crate::runtime::Runtime;
    
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
        let runtime = Runtime::init_local();

        runtime.spawn_local(async {
            Executor::spawn_local(async {
                let result = dummy_async().await;
                assert!(result.is_ok());
            });
        });

        runtime.run();
    }
}
