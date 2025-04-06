mod app;
mod callback;
mod color;
mod cursor;
mod error;
mod layout;
mod renderer;
mod element;
mod signal;
mod tree;
mod view;

pub use app::launch;
pub use color::*;
pub use element::Element;
pub use signal::Signal;
pub use view::*;
pub use error::Error;
pub use layout::{Alignment, HAlignment, VAlignment, Orientation};

pub type AppResult = Result<(), Error>;
