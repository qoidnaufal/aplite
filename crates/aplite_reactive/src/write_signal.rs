use std::marker::PhantomData;
use aplite_storage::Key;

use crate::runtime::ReactiveId;
use super::traits::*;

#[derive(Clone, Copy)]
pub struct SignalWrite<T> {
    pub(crate) id: Key<ReactiveId>,
    pub(crate) phantom: PhantomData<T>
}

impl<T: 'static> SignalWrite<T> {
    pub(crate) fn new(id: Key<ReactiveId>) -> Self {
        Self { id, phantom: PhantomData }
    }
}

impl<T: 'static> Reactive for SignalWrite<T> {
    fn id(&self) -> Key<ReactiveId> { self.id }
}

impl<T: 'static> Notify for SignalWrite<T> {
    type Value = T;
}
