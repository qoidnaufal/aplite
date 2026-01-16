mod app;
mod callback;
mod context;
mod cursor;
mod error;
mod layout;
mod state;
mod view;
mod widget;

pub mod prelude {
    // pub use winit::window::{WindowAttributes, WindowButtons, WindowLevel};
    // pub use winit::dpi::{Size, LogicalSize, PhysicalSize};
    // pub use winit::dpi::{Position, LogicalPosition, PhysicalPosition};

    pub use aplite_reactive::*;
    pub use aplite_renderer::Shape;
    pub use aplite_types::CornerRadius;
    pub use aplite_types::Length::{Fixed, Grow, FitContent};

    pub use crate::app::{Aplite, AppConfig, Launch};
    pub use crate::context::Context;
    pub use crate::cursor::Cursor;

    pub use crate::layout::{
        Axis,
        Padding,
        AlignV,
        AlignH
    };

    pub use crate::callback::WidgetEvent;

    pub use crate::widget::*;

    pub use crate::view::{
        IntoView,
        ToAnyView,
    };

    pub type ApliteResult = Result<(), crate::error::ApliteError>;
}

pub mod color {
    pub use aplite_types::theme;
    pub use aplite_types::{Color, rgb, rgba};
}
