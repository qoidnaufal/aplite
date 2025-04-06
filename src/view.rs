mod button;
mod image;
mod stack;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::layout::LayoutCtx;
use crate::tree::WidgetTree;
use crate::renderer::{Gfx, Gpu};
use crate::element::{Attributes, Element, Shape, Style};
use crate::{Orientation, Pixel, Rgba};
use crate::callback::CALLBACKS;

pub use {
    button::*,
    image::*,
    stack::*,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new() -> Self {
        static NODE_ID: AtomicU64 = AtomicU64::new(0);
        Self(NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type AnyView = Box<dyn View>;

impl std::fmt::Debug for AnyView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id())
    }
}

pub trait View {
    fn id(&self) -> NodeId;
    fn element(&self) -> Element;
    fn children(&self) -> Option<&[AnyView]>;
    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>>;
    fn style(&self) -> Style;
    fn layout(&self, cx: &mut LayoutCtx) -> Attributes;

    fn build_tree(&self, tree: &mut WidgetTree) {
        let node_id = self.id();
        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                tree.insert_children(node_id, child.id());
                tree.insert_parent(child.id(), node_id);
                child.build_tree(tree);
            });
        }
    }

    fn calculate_dimensions(&self, cx: &mut LayoutCtx) {
        let style = self.style();
        let mut size = style.dimensions();
        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                child.calculate_dimensions(cx);
                let child_size = cx.get_attributes(&child.id()).dims;
                match style.orientation() {
                    Orientation::Vertical => {
                        size.height += child_size.height;
                        size.width = size.width.max(child_size.width + style.padding() * 2);
                    }
                    Orientation::Horizontal => {
                        size.height = size.height.max(child_size.height + style.padding() * 2);
                        size.width += child_size.width - 1;
                    }
                }
            });
            let child_len = children.len() as u32;
            match style.orientation() {
                Orientation::Vertical => {
                    size.height += style.padding() * 2 + style.spacing() * (child_len - 1);
                },
                Orientation::Horizontal => {
                    size.width += style.padding() * 2 + style.spacing() * (child_len - 1);
                },
            }
        }
        cx.insert_attributes(self.id(), size);
    }

    fn prepare(
        &self,
        gpu: &Gpu,
        gfx: &mut Gfx,
        tree: &mut WidgetTree,
    ) {
        let node_id = self.id();
        if tree.is_root(&node_id) {
            self.build_tree(tree);
            self.calculate_dimensions(&mut tree.layout);
        }
        let attr = self.layout(&mut tree.layout);
        let half = attr.dims / 2;
        let current_pos = attr.pos;
        let mut element = self.element();
        tree.nodes.push(node_id);
        tree.cached_color.insert(node_id, element.fill_color());
        gfx.push_texture(gpu, self.pixel(), &mut element);
        gfx.register(element, &attr, gpu.size());

        if let Some(children) = self.children() {
            let padding = tree.layout.padding(&node_id);
            tree.layout.set_next_pos(|pos| {
                pos.x = current_pos.x - half.width + padding;
                pos.y = current_pos.y - half.height + padding;
            });

            children.iter().for_each(|child| {
                child.prepare(gpu, gfx, tree);
            });

            if let Some(parent_id) = tree.get_parent(&node_id) {
                tree.layout.reset_to_parent(*parent_id, current_pos, half);
            }
        }

    }
}

pub trait IntoView: Sized {
    type V: View + 'static;
    fn into_view(self) -> Self::V;
    fn into_any(self) -> AnyView { Box::new(self.into_view()) }
}

pub struct DynView(AnyView);

impl View for DynView {
    fn id(&self) -> NodeId { self.0.id() }

    fn element(&self) -> Element {
        self.0.element()
    }

    fn children(&self) -> Option<&[AnyView]> { self.0.children() }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { self.0.pixel() }

    fn style(&self) -> Style { self.0.style() }

    fn layout(&self, cx: &mut LayoutCtx) -> Attributes {
        self.0.layout(cx)
    }
}

impl<F, IV> IntoView for F
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static
{
    type V = DynView;
    fn into_view(self) -> Self::V {
        let any_view = self().into_any();
        DynView(any_view)
    }
}

pub struct TestTriangleWidget {
    id: NodeId,
    style: Style,
}

impl TestTriangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let style = Style::new(Rgba::RED, (300, 300), Shape::Triangle);
        Self { id, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
    }

    pub fn on_hover<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for TestTriangleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn element(&self) -> Element { Element::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx) -> Attributes {
        // cx.insert_attributes(self.id, self.style.dimensions());
        cx.assign_position(&self.id)
    }

    fn style(&self) -> Style { self.style }
}

impl IntoView for TestTriangleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}

pub struct TestCircleWidget {
    id: NodeId,
    style: Style,
}

impl TestCircleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let style = Style::new(Rgba::RED, (300, 300), Shape::Circle);
        Self { id, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
    }

    pub fn on_hover<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for TestCircleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn element(&self) -> Element { Element::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx) -> Attributes {
        // cx.insert_attributes(self.id, self.style.dimensions());
        cx.assign_position(&self.id)
    }

    fn style(&self) -> Style { self.style }
}

impl IntoView for TestCircleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
