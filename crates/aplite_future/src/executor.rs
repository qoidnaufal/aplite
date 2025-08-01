use std::sync::{Arc, RwLock};

use crate::runtime::CURRENT_RUNTIME;
use crate::task::Task;

pub struct Executor;

impl Executor {
    pub fn spawn_local(future: impl Future<Output = ()> + 'static) {
        CURRENT_RUNTIME.with(|cell| {
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
