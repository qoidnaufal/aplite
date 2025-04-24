mod callback;
mod renderer;
mod cursor;

pub mod app;
pub mod color;
pub mod element;
pub mod error;
pub mod layout;
pub mod reactive;
pub mod properties;
pub mod context;
pub mod view;

pub mod prelude {
    use crate::error::GuiError;

    pub use crate::app::App;
    pub use crate::reactive::{arc_signal, signal, Get, Set};
    pub use crate::color::Rgba;
    pub use crate::element::Element;
    pub use crate::properties::Orientation;
    pub use crate::view::{
        IntoView,
        TestCircleWidget,
        TestTriangleWidget,
        stack,
        button,
        image
    };

    pub type AppResult = Result<(), GuiError>;

    // pub fn launch<F, IV>(f: F) -> Result<(), Error>
    // where
    //     F: Fn() -> IV + 'static,
    //     IV: IntoView + 'static,
    // {
    //     App::new(f).launch()
    // }
}
