use std::marker::PhantomData;

use super::reactive_traits::*;

use crate::signal_read::SignalRead;
use crate::graph::{ReactiveId, GRAPH};
use crate::signal_write::SignalWrite;

#[derive(Clone, Copy)]
pub struct Signal<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        GRAPH.with(|rt| rt.create_signal(value))
    }

    pub fn split(value: T) -> (SignalRead<T>, SignalWrite<T>) {
        GRAPH.with(|rt| rt.create_signal(value)).into_split()
    }

    pub fn into_split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (
            SignalRead { id: self.id, phantom: PhantomData },
            SignalWrite { id: self.id, phantom: PhantomData }
        )
    }

    pub fn read_only(&self) -> SignalRead<T> {
        SignalRead::new(self.id)
    }

    pub fn write_only(&self) -> SignalWrite<T> {
        SignalWrite::new(self.id)
    }
}

impl<T: 'static> Reactive for Signal<T> {
    fn id(&self) -> &ReactiveId { &self.id }
}

impl<T: 'static> Track for Signal<T> {}

impl<T: 'static> Read for Signal<T> {
    type Value = T;
}

impl<T: 'static> Notify for Signal<T> {}

impl<T: 'static> Write for Signal<T> {
    type Value = T;
}

impl<T, R: Reactive> PartialEq<R> for Signal<T> {
    fn eq(&self, other: &R) -> bool {
        self.id.eq(other.id())
    }
}

impl<T, R: Reactive> PartialOrd<R> for Signal<T> {
    fn partial_cmp(&self, other: &R) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(other.id())
    }
}

impl<T: 'static> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
