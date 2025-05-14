mod app;
mod image_data;
mod context;
mod error;
mod properties;
mod reactive;
mod renderer;
mod tree;
mod view;

pub mod prelude {
    use crate::error::ApliteError;

    pub use shared::Rgba;
    pub use crate::app::Aplite;
    pub use crate::context::Context;
    pub use crate::properties::AspectRatio;
    pub use crate::image_data::image_reader;
    pub use crate::reactive::{arc_signal, signal, Get, Set};
    pub use crate::renderer::element::Shape;
    pub use crate::context::layout::{
        Orientation,
        Alignment,
        VAlign,
        HAlign
    };
    pub use crate::view::{
        View,
        TestCircleWidget,
        // TestTriangleWidget,
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

    pub type ApliteResult = Result<(), ApliteError>;
}
