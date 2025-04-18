use super::SignalId;

pub trait Reactive {
    fn id(&self) -> SignalId;
}

pub trait Get: Reactive {
    type Value: Clone;
    fn get(&self) -> Self::Value;
}

pub trait Set: Reactive {
    type Value;
    fn set(&self, f: impl FnOnce(&mut Self::Value));
}
