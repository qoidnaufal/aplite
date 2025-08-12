mod app;
mod context;
mod error;
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
    pub use crate::state::{WidgetState, AspectRatio};
    pub use crate::context::layout::{
        Orientation,
        Padding,
        AlignV,
        AlignH
    };
    pub use crate::widget::{
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
        ViewNode,
    };
    pub use crate::widget::WidgetEvent::*;

    pub type ApliteResult = Result<(), ApliteError>;
}
