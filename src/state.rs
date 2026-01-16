#[derive(Debug, Clone, Copy)]
pub struct BorderWidth(pub(crate) f32);

#[derive(Debug, Clone, Copy)]
pub struct Radius(pub(crate) f32);

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
partial_eq!(Radius);

// pub trait Properties {}

// macro_rules! impl_tuple_macro {
//     ($macro:ident, $next:tt) => {
//         $macro!{$next}
//     };
//     ($macro:ident, $next:tt, $($rest:tt),*) => {
//         $macro!{$next, $($rest),*}
//         impl_tuple_macro!{$macro, $($rest),*}
//     };
// }
