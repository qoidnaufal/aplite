mod button;
mod image;
mod stack;

use crate::layout::{Attributes, Layout};
use crate::style::{HAlign, Orientation, Shape, Style, VAlign};
use crate::tree::{NodeId, WidgetTree};
use crate::renderer::{Gfx, Gpu};
use crate::element::Element;
use crate::color::{Pixel, Rgba};
use crate::callback::CALLBACKS;

pub use {
    button::*,
    image::*,
    stack::*,
};

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
    fn layout(&self, layout: &mut Layout) -> Attributes;

    fn calculate_size(&self, layout: &mut Layout) {
        let style = self.style();
        let padding = style.padding();
        let mut size = style.size();

        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                child.calculate_size(layout);
                let child_size = layout.get_attributes(&child.id()).size;
                match style.orientation() {
                    Orientation::Vertical => {
                        size.height += child_size.height;
                        size.width = size.width.max(child_size.width + padding.horizontal());
                    }
                    Orientation::Horizontal => {
                        size.height = size.height.max(child_size.height + padding.vertical());
                        size.width += child_size.width - 1;
                    }
                }
            });
            let child_len = children.len() as u32;
            let stretch = style.spacing() * (child_len - 1);
            match style.orientation() {
                Orientation::Vertical => {
                    size.height += padding.vertical() + stretch;
                },
                Orientation::Horizontal => {
                    size.width += padding.horizontal() + stretch;
                },
            }
        }

        let final_size = size
            .max(style.min_width(), style.min_height())
            .min(style.max_width(), style.max_height());
        layout.insert_attributes(self.id(), final_size);
    }

    fn prepare(
        &self,
        gpu: &Gpu,
        gfx: &mut Gfx,
        tree: &mut WidgetTree,
    ) {
        let node_id = self.id();
        tree.nodes.push(
            node_id,
            self.children().map(|slice| slice.iter().map(|av| av.id()).collect()),
            self.style()
        );
        if tree.is_root(&node_id) {
            self.calculate_size(&mut tree.layout);
        }
        let attr = self.layout(&mut tree.layout); // FIXME: this is sucks
        let current_half = attr.size / 2;
        let current_pos = attr.pos;
        let mut element = self.element();
        tree.cached_color.insert(node_id, element.fill_color());
        gfx.push_texture(gpu, self.pixel(), &mut element);
        gfx.register(element, &attr, gpu.size());

        if let Some(children) = self.children() {
            let padding = tree.layout.get_padding(&node_id);
            // let alignment = tree.layout.alignment();

            // setting for the first child's position
            tree.layout.set_next_pos(|pos| {
                // match (alignment.horizontal, alignment.vertical) {
                //     (HAlignment::Left, VAlignment::Top) => {
                //         pos.x = current_pos.x - current_half.width + padding.left();
                //         pos.y = current_pos.y - current_half.height + padding.top();
                //     },
                //     (HAlignment::Left, VAlignment::Middle) => {},
                //     (HAlignment::Left, VAlignment::Bottom) => {},
                //     (HAlignment::Center, VAlignment::Top) => {},
                //     (HAlignment::Center, VAlignment::Middle) => {},
                //     (HAlignment::Center, VAlignment::Bottom) => {},
                //     (HAlignment::Right, VAlignment::Top) => {},
                //     (HAlignment::Right, VAlignment::Middle) => {},
                //     (HAlignment::Right, VAlignment::Bottom) => {},
                // }

                pos.x = current_pos.x - current_half.width + padding.left();
                pos.y = current_pos.y - current_half.height + padding.top();
            });

            children.iter().for_each(|child| {
                child.prepare(gpu, gfx, tree);
            });

            if let Some(parent_id) = tree.get_parent(&node_id) {
                tree.layout.reset_to_parent(*parent_id, current_pos, current_half);
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

    fn layout(&self, layout: &mut Layout) -> Attributes {
        self.0.layout(layout)
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

    pub fn style<F: FnOnce(&mut Style)>(mut self, f: F) -> Self {
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

    fn layout(&self, layout: &mut Layout) -> Attributes {
        layout.assign_position(&self.id)
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

    fn layout(&self, layout: &mut Layout) -> Attributes {
        layout.assign_position(&self.id)
    }

    fn style(&self) -> Style { self.style }
}

impl IntoView for TestCircleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
