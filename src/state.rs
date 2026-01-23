#[derive(Debug, Clone, Copy)]
pub struct BorderWidth(pub f32);

macro_rules! partial_eq {
    ($name:ident) => {
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for $name {}
    };
}

partial_eq!(BorderWidth);
