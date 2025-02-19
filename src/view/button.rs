use crate::context::{Alignment, LayoutCtx};
use crate::shapes::{Shape, ShapeKind};
use crate::callback::CALLBACKS;
use crate::Rgb;
use super::{AnyView, IntoView, NodeId, View};

pub fn button() -> Button { Button::new() }

pub struct Button {
    id: NodeId,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId { self.id }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::RED, ShapeKind::FilledRectangle, (120, 40))
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }
}

impl View for Button {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape { self.shape() }

    fn img_src(&self) -> Option<&std::path::PathBuf> { None }

    fn layout(&self, cx: &mut LayoutCtx, shape: &mut Shape) {
        cx.assign_position(shape);
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
