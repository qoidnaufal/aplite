mod app;
mod context;
mod error;
mod reactive;
mod renderer;
mod view;

pub mod prelude {
    use crate::error::ApliteError;

    pub use shared::Rgba;
    pub use crate::app::Aplite;
    pub use crate::context::Context;
    pub use crate::context::properties::AspectRatio;
    pub use crate::reactive::{arc_signal, signal, Get, Set};
    pub use crate::renderer::texture::image_reader;
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

    pub use winit::window::Fullscreen;
}
