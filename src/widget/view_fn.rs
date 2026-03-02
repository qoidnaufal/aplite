use crate::view::IntoView;
use crate::widget::Widget;
use crate::context::{BuildCx, LayoutCx, CursorCx};

impl<F, IV> IntoView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    type View = ViewFn<F, IV>;

    fn into_view(self) -> Self::View {
        ViewFn::new(self)
    }
}

pub struct ViewFn<F: Fn() -> IV, IV: IntoView> {
    view_fn: F,
}

impl<F, IV> ViewFn<F, IV>
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    pub fn new(view_fn: F) -> Self {
        Self {
            view_fn,
        }
    }
}

impl<F, IV> Widget for ViewFn<F, IV>
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    fn debug_name(&self) -> &'static str {
        (self.view_fn)().into_view().debug_name()
    }

    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        (self.view_fn)().into_view().build(cx)
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        (self.view_fn)().into_view().layout(cx)
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        (self.view_fn)().into_view().detect_hover(cx)
    }
}

impl<F, IV> IntoView for ViewFn<F, IV>
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
{
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl<F, IV> std::fmt::Display for ViewFn<F, IV>
where
    F: Fn() -> IV + 'static,
    IV: IntoView,
    IV::View: Widget + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.view_fn)().into_view().fmt(f)
    }
}
