use std::marker::PhantomData;

use super::traits::*;
use crate::read_signal::SignalRead;
use crate::runtime::{ReactiveId, RUNTIME};
use crate::write_signal::SignalWrite;

#[derive(Clone, Copy)]
pub struct RwSignal<T> {
    pub(crate) id: ReactiveId,
    pub(crate) phantom: PhantomData<T>
}

impl<T: 'static> RwSignal<T> {
    pub fn new(value: T) -> Self {
        RUNTIME.with(|rt| rt.create_rw_signal(value))
    }

    pub fn split(self) -> (SignalRead<T>, SignalWrite<T>) {
        (SignalRead::new(self.id), SignalWrite::new(self.id))
    }

    pub fn read_only(&self) -> SignalRead<T> { SignalRead::new(self.id) }

    pub fn write_only(&self) -> SignalWrite<T> { SignalWrite::new(self.id) }
}

impl<T: 'static> Reactive for RwSignal<T> {
    fn id(&self) -> ReactiveId { self.id }
}

impl<T: 'static> Track for RwSignal<T> {
    type Value = T;
}

impl<T: 'static> Notify for RwSignal<T> {
    type Value = T;
}
