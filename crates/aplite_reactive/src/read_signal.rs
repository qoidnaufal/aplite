use std::marker::PhantomData;

use crate::graph::ReactiveId;
use crate::reactive_traits::*;

#[derive(Clone, Copy)]
pub struct ReadSignal<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: 'static> ReadSignal<T> {
    pub(crate) fn new(id: ReactiveId) -> Self {
        Self { id, phantom: PhantomData }
    }
}

impl<T: 'static> Reactive for ReadSignal<T> {
    fn id(&self) -> &ReactiveId { &self.id }
}

impl<T: 'static> Track for ReadSignal<T> {}

impl<T: 'static> Read for ReadSignal<T> {
    type Value = T;
}

impl<T, R: Reactive> PartialEq<R> for ReadSignal<T> {
    fn eq(&self, other: &R) -> bool {
        self.id.eq(other.id())
    }
}

impl<T, R: Reactive> PartialOrd<R> for ReadSignal<T> {
    fn partial_cmp(&self, other: &R) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(other.id())
    }
}

impl<T: 'static> std::fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("id", self.id())
            .finish()
    }
}
