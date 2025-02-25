use crate::context::{Alignment, LayoutCtx};
use crate::renderer::{Gfx, Gpu};
use crate::shapes::{Shape, ShapeConfig, ShapeKind};
use crate::callback::CALLBACKS;
use crate::Rgb;
use super::{AnyView, Configs, IntoView, NodeId, View};

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

    fn config(&self, _gpu: &Gpu, _gfx: &mut Gfx, configs: &mut Configs) {
        let config = ShapeConfig::new((120, 40), Rgb::RED);
        configs.insert(self.id, config);
        // Shape::filled(Rgb::RED, ShapeKind::RoundedRect, (120, 40))
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

    fn config(&self, gpu: &Gpu, gfx: &mut Gfx, configs: &mut Configs) {
        self.config(gpu, gfx, configs);
    }

    fn img_src(&self) -> Option<&std::path::PathBuf> { None }

    fn shape_kind(&self) -> ShapeKind { ShapeKind::RoundedRect }

    fn layout(&self, cx: &mut LayoutCtx, config: &mut ShapeConfig) {
        cx.assign_position(config);
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for Button {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
