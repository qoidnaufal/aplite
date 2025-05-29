use std::marker::PhantomData;
use aplite_storage::Key;

use crate::runtime::ReactiveId;
use crate::traits::*;

#[derive(Clone, Copy)]
pub struct SignalRead<T> {
    pub(crate) id: Key<ReactiveId>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: 'static> SignalRead<T> {
    pub(crate) fn new(id: Key<ReactiveId>) -> Self {
        Self { id, phantom: PhantomData }
    }
}

impl<T: 'static> Reactive for SignalRead<T> {
    fn id(&self) -> Key<ReactiveId> { self.id }
}

impl<T: 'static> Track for SignalRead<T> {
    type Value = T;
}
