use crate::runtime::ReactiveId;

pub trait Reactive {
    type Value: 'static;
    fn id(&self) -> ReactiveId;
}

pub trait Get: Reactive {
    fn get(&self) -> Self::Value;
}

pub trait Set: Reactive {
    fn set(&self, value: Self::Value);
}

pub trait With: Reactive {
    fn with<R, F: FnOnce(&Self::Value) -> R>(&self, f: F) -> R;
}

pub trait Update: Reactive {
    fn update(&self, f: impl FnOnce(&mut Self::Value));
}
