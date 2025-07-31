use aplite_renderer::Shape;
use aplite_types::{CornerRadius, Rgba, Size};
// use aplite_reactive::*;

use crate::{view::VIEW_STORAGE, widget::Widget};

/// this is just a wrapper over `FnMut(Option<T>) -> T`
pub trait FnEl<T>: FnMut(Option<T>) -> T {}

impl<F, T> FnEl<T> for F where F: FnMut(Option<T>) -> T {}

/// this is just a wrapper over `FnMut() -> T`
pub trait FnAction<T>: FnMut() -> T {}

impl<F, T> FnAction<T> for F where F: FnMut() -> T {}

impl<T> Style for T where T: Widget + Sized {}

// FIXME: there are too many effects here, maybe shouldn't
/// trait to modify the rendered element
pub trait Style: Widget + Sized {
    fn set_color<F>(self, f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static,
    {
        let _ = f;
        self
    }

    fn set_stroke_color<F>(self, f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static
    {
        let _ = f;
        self
    }

    fn set_hover_color<F>(self, f: F) -> Self
    where
        F: FnAction<Rgba<u8>> + 'static
    {
        let _ = f;
        self
    }

    fn set_click_color<F>(self, f: F) -> Self
    where
        F: FnAction<Rgba<u8>> + 'static,
    {
        let _ = f;
        self
    }

    fn set_stroke_width<F>(self, f: F) -> Self
    where
        F: FnEl<u32> + 'static
    {
        let _ = f;
        self
    }

    fn set_rotation<F>(self, f: F) -> Self
    where
        F: FnEl<f32> + 'static
    {
        let _ = f;
        self
    }

    fn set_corners<F>(self, f: F) -> Self
    where
        F: FnEl<CornerRadius> + 'static
    {
        let _ = f;
        self
    }

    fn set_shape<F>(self, f: F) -> Self
    where
        F: FnEl<Shape> + 'static
    {
        let _ = f;
        self
    }

    fn set_size(self, size: impl Into<Size>) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_mut(&self.id()).unwrap();
            state.rect.set_size(size.into());
        });
        self
    }

    fn set_dragable(self, value: bool) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_mut(&self.id()).unwrap();
            state.dragable = value;
        });
        self
    }
}
