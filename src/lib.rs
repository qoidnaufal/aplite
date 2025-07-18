mod app;
mod context;
mod error;
mod view;
mod widget_state;

pub mod prelude {
    use crate::error::ApliteError;

    pub use aplite_reactive::*;
    pub use aplite_renderer::image_reader;
    pub use aplite_renderer::Shape;
    pub use aplite_types::{Rgba, CornerRadius};

    pub use crate::app::Aplite;
    pub use crate::context::Context;
    pub use crate::widget_state::AspectRatio;
    pub use crate::context::layout::{
        Alignment,
        Orientation,
        Padding,
        AlignV,
        AlignH
    };
    pub use crate::view::{
        Widget,
        IntoView,
        View,
        Style,
        Layout,
        ViewNode,
        CircleWidget,
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
