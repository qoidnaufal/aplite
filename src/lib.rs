mod callback;
mod context;
mod cursor;
mod renderer;
mod tree;

pub mod app;
pub mod color;
pub mod error;
pub mod layout;
pub mod reactive;
pub mod properties;
pub mod view;

pub mod prelude {
    use crate::error::GuiError;

    pub use crate::app::App;
    pub use crate::reactive::{arc_signal, signal, Get, Set};
    pub use crate::color::Rgba;
    pub use crate::context::Context;
    pub use crate::properties::{Orientation, Shape};
    pub use crate::view::{
        View,
        TestCircleWidget,
        TestTriangleWidget,
        HStack,
        VStack,
        Button,
        Image,
    };
    pub use crate::view::{
        h_stack,
        v_stack,
        button,
        image,
    };

    pub type AppResult = Result<(), GuiError>;
}
