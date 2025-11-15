mod app;
mod context;
mod cursor;
mod error;
mod layout;
mod state;
mod view;
mod widget;

pub type ApliteResult = Result<(), crate::error::ApliteError>;

pub mod prelude {
    pub use aplite_reactive::*;
    pub use aplite_renderer::Shape;
    pub use aplite_types::CornerRadius;
    pub use aplite_types::Unit::{Fixed, Grow, Fit};

    pub use crate::app::{Aplite, AppConfig};
    pub use crate::context::Context;
    pub use crate::cursor::Cursor;
    pub use crate::state::AspectRatio;

    pub use crate::layout::{
        Orientation,
        Padding,
        AlignV,
        AlignH
    };

    pub use crate::widget::{
        Widget,
        InteractiveWidget,
        ParentWidget,
        image_reader,
    };

    pub use crate::widget::{
        circle,  CircleWidget,
        h_stack, HStack,
        v_stack, VStack,
        button,  Button,
        image,   Image,
    };

    pub use crate::view::{
        IntoView,
        View,
    };

    pub use crate::view::WidgetEvent::*;
}

pub mod color {
    pub use aplite_types::theme;
    pub use aplite_types::{Rgba, rgba8, rgba32, rgba_hex, rgba_hex_alpha};
}
