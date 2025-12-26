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
    pub use aplite_types::Length::{Fixed, Grow, Fit};

    pub use crate::app::{Aplite, AppConfig, Run};
    pub use crate::context::Context;
    pub use crate::cursor::Cursor;

    pub use crate::layout::{
        Orientation,
        Padding,
        AlignV,
        AlignH
    };

    pub use crate::callback::WidgetEvent;

    pub use crate::widget::*;

    pub use crate::view::{
        IntoView,
        ViewTuple,
        View,
    };

    pub type ApliteResult = Result<(), crate::error::ApliteError>;
}

pub mod color {
    pub use aplite_types::theme;
    pub use aplite_types::{Rgba, rgba8, rgba32, hex, hex_alpha};
}
