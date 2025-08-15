mod app;
mod context;
mod cursor;
mod error;
mod layout;
mod state;
mod view;
mod widget;

pub mod prelude {
    use crate::error::ApliteError;

    pub use aplite_reactive::*;
    pub use aplite_renderer::Shape;
    pub use aplite_types::{Rgba, rgba_u8, rgba_f32, rgba_hex, CornerRadius};

    pub use crate::app::Aplite;
    pub use crate::context::Context;
    pub use crate::cursor::Cursor;
    pub use crate::state::{ViewNode, AspectRatio};
    pub use crate::layout::{
        Orientation,
        Padding,
        AlignV,
        AlignH
    };
    pub use crate::widget::{
        WidgetId,
        Widget,
        WidgetExt,
        CircleWidget,
        HStack,
        VStack,
        Button,
        Image,
        h_stack,
        v_stack,
        button,
        image,
        image_reader,
    };
    pub use crate::view::{
        IntoView,
        View,
    };
    pub use crate::widget::WidgetEvent::*;

    pub type ApliteResult = Result<(), ApliteError>;
}
