use aplite_storage::Table;
use aplite_renderer::Shape;
use aplite_types::{
    Matrix3x2,
    Rect,
    // Size,
    CornerRadius,
    Paint,
    Rgba,
    Unit,
};

#[derive(Debug, Clone, Copy)]
pub enum AspectRatio {
    Defined(u8, u8),
    Source,
    Undefined,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag {
    pub(crate) visible: bool,
    pub(crate) focusable: bool,
    pub(crate) hoverable: bool,
    pub(crate) movable: bool,
    pub(crate) needs_redraw: bool,
}

impl Default for Flag {
    fn default() -> Self {
        Self {
            visible: true,
            focusable: false,
            hoverable: false,
            movable: false,
            needs_redraw: true,
        }
    }
}

#[derive(Clone)]
pub struct Border {
    pub(crate) paint: Paint,
    pub(crate) width: f32,
}

pub struct State {
    table: Table,
}
