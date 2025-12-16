use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::Wake;
use std::future::Future;

use crate::executor::SPAWNER;

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
