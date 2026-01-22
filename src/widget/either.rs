use aplite_reactive::{Memo, Get};

use crate::widget::Widget;
use crate::view::IntoView;
use crate::context::{BuildCx, LayoutCx, CursorCx};

pub fn either<W, TrueFn, FalseFn, VT, VF>(
    when: W,
    content_true: TrueFn,
    content_false: FalseFn,
) -> impl IntoView
where
    W: Fn() -> bool + 'static,
    TrueFn: Fn() -> VT + 'static,
    FalseFn: Fn() -> VF + 'static,
    VT: IntoView,
    VF: IntoView,
{
    let when = Memo::new(move |_| when());

    let f = move || match when.get() {
        true => Either::True(content_true()),
        false => Either::False(content_false()),
    };

    ViewFn::new(f)
}

/*
#########################################################
#
# ViewFn
#
#########################################################
*/

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

/*
#########################################################
#
# Either
#
#########################################################
*/

enum Either<VT, VF> {
    True(VT),
    False(VF)
}

impl<VT, VF> Widget for Either<VT, VF>
where
    VT: Widget,
    VF: Widget,
{
    fn debug_name(&self) -> &'static str {
        match self {
            Either::True(t) => t.debug_name(),
            Either::False(f) => f.debug_name(),
        }
    }

    fn build(&self, cx: &mut BuildCx<'_>) -> bool {
        match self {
            Either::True(t) => t.build(cx),
            Either::False(f) => f.build(cx),
        }
    }

    fn layout(&self, cx: &mut LayoutCx<'_>) {
        match self {
            Either::True(t) => t.layout(cx),
            Either::False(f) => f.layout(cx),
        }
    }

    fn detect_hover(&self, cx: &mut CursorCx<'_>) -> bool {
        match self {
            Either::True(t) => t.detect_hover(cx),
            Either::False(f) => f.detect_hover(cx),
        }
    }
}

impl<VT, VF> IntoView for Either<VT, VF>
where
    VT: IntoView,
    VF: IntoView,
{
    type View = Either<VT::View, VF::View>;

    fn into_view(self) -> Self::View {
        match self {
            Either::True(t) => Either::True(t.into_view()),
            Either::False(f) => Either::False(f.into_view()),
        }
    }
}
