mod app;
mod context;
mod error;
mod widget_state;
mod view;
mod widget;
mod style;

pub mod prelude {
    use crate::error::ApliteError;

    pub use aplite_reactive::*;
    pub use aplite_renderer::Shape;
    pub use aplite_types::{Rgba, rgba_u8, rgba_f32, rgba_hex, CornerRadius};

    pub use crate::app::Aplite;
    pub use crate::context::Context;
    pub use crate::widget_state::AspectRatio;
    pub use crate::style::{Style, FnEl, FnAction};
    pub use crate::context::layout::{
        Alignment,
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
