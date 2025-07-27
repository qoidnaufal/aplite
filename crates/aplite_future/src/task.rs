use std::pin::Pin;
use std::sync::{Arc, RwLock};
// use std::sync::mpsc::Sender;
use std::task::Wake;

use crate::runtime::WeakSender;

type PinnedFuture = Pin<Box<dyn Future<Output = ()>>>;

pub(crate) struct Task {
    pub(crate) future: RwLock<Option<PinnedFuture>>,
    pub(crate) sender: WeakSender,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let cloned = Arc::clone(&self);
        if let Some(sender) = self.sender.upgrade() {
            let _ = sender.send(cloned);
        }
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}
