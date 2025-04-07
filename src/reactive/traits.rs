pub trait Get {
    type Value: Clone;
    fn get(&self) -> Self::Value;
}

pub trait Set {
    type Value;
    fn set(&self, f: impl FnOnce(&mut Self::Value));
}
