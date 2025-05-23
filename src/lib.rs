mod app;
mod context;
mod error;
mod reactive;
mod view;

pub mod prelude {
    use crate::error::ApliteError;

    pub use aplite_types::Rgba;
    pub use aplite_renderer::image_reader;
    pub use aplite_renderer::Shape;

    pub use crate::app::Aplite;
    pub use crate::context::Context;
    pub use crate::context::properties::AspectRatio;
    pub use crate::reactive::{arc_signal, signal, Get, Set};
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

    pub use winit::window::Fullscreen;
}
