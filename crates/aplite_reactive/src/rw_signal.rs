use std::marker::PhantomData;

use super::reactive_traits::*;
use crate::read_signal::ReadSignal;
use crate::graph::{ReactiveId, GRAPH};
use crate::write_signal::WriteSignal;

#[derive(Clone, Copy)]
pub struct RwSignal<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>
}

impl<T: 'static> RwSignal<T> {
    pub fn new(value: T) -> Self {
        GRAPH.with(|rt| rt.create_rw_signal(value))
    }

    pub fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (ReadSignal::new(self.id), WriteSignal::new(self.id))
    }

    pub fn read_only(&self) -> ReadSignal<T> { ReadSignal::new(self.id) }

    pub fn write_only(&self) -> WriteSignal<T> { WriteSignal::new(self.id) }
}

impl<T: 'static> Reactive for RwSignal<T> {
    fn id(&self) -> &ReactiveId { &self.id }
}

impl<T: 'static> Track for RwSignal<T> {}

impl<T: 'static> Read for RwSignal<T> {
    type Value = T;
}

impl<T: 'static> Notify for RwSignal<T> {}

impl<T: 'static> Write for RwSignal<T> {
    type Value = T;
}

impl<T, R: Reactive> PartialEq<R> for RwSignal<T> {
    fn eq(&self, other: &R) -> bool {
        self.id.eq(other.id())
    }
}

impl<T, R: Reactive> PartialOrd<R> for RwSignal<T> {
    fn partial_cmp(&self, other: &R) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(other.id())
    }
}

impl<T: 'static> std::fmt::Debug for RwSignal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("id", self.id())
            .finish()
    }
}
