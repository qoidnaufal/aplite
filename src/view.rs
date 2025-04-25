mod button;
mod image;
mod stack;

use crate::properties::{Orientation, Shape, Properties, HAlign, VAlign};
use crate::context::{NodeId, Context};
use crate::renderer::{Gfx, Gpu};
use crate::color::{Pixel, Rgba};

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
    fn children(&self) -> Option<&[AnyView]>;
    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>>;
    fn properties(&self) -> &Properties;
    fn layout(&self, cx: &mut Context);

    fn calculate_size(&self, cx: &mut Context) {
        let prop = self.properties();
        let padding = prop.padding();
        let mut size = prop.size();

        if let Some(children) = self.children() {
            children.iter().for_each(|child| {
                child.calculate_size(cx);
                let child_size = cx.get_node_data(child.id()).size();
                match prop.orientation() {
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
            let stretch = prop.spacing() * (child_len - 1);
            match prop.orientation() {
                Orientation::Vertical => {
                    size.height += padding.vertical() + stretch;
                },
                Orientation::Horizontal => {
                    size.width += padding.horizontal() + stretch;
                },
            }
        }

        let final_size = size
            .max(prop.min_width(), prop.min_height())
            .min(prop.max_width(), prop.max_height());

        let properties = cx.get_node_data_mut(self.id());
        properties.set_size(final_size);
        // if size != final_size {
        // }
    }

    fn prepare(&self, cx: &mut Context, parent_id: Option<NodeId>) {
        let node_id = self.id();
        cx.insert(node_id, parent_id, self.properties());
        if let Some(children) = self.children() {
            children.iter().for_each(|child_view| child_view.prepare(cx, Some(node_id)));
        }
    }

    fn render(
        &self,
        gpu: &Gpu,
        gfx: &mut Gfx,
        cx: &mut Context,
    ) {
        let node_id = self.id();
        if cx.is_empty() {
            self.prepare(cx, None);
            self.calculate_size(cx);
        }
        self.layout(cx);
        let properties = cx.get_node_data(node_id);
        gfx.register(gpu, self.pixel(), properties);

        if let Some(children) = self.children() {
            let padding = properties.padding();
            let current_pos = properties.pos();
            let current_half = properties.size() / 2;

            // FIXME: consider alignment
            cx.set_next_pos(|pos| {
                pos.x = current_pos.x - current_half.width + padding.left();
                pos.y = current_pos.y - current_half.height + padding.top();
            });

            children.iter().for_each(|child| {
                child.render(gpu, gfx, cx);
            });

            if let Some(parent_idx) = cx.get_parent(&node_id) {
                cx.reset_to_parent(*parent_idx, current_pos, current_half);
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

    fn children(&self) -> Option<&[AnyView]> { self.0.children() }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { self.0.pixel() }

    fn properties(&self) -> &Properties { self.0.properties() }

    fn layout(&self, cx: &mut Context) {
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
    inner: Properties,
}

impl TestTriangleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let inner = Properties::new(Rgba::RED, (300, 300), Shape::Triangle, false);
        Self { id, inner }
    }

    pub fn style<F: FnOnce(&mut Properties)>(mut self, f: F) -> Self {
        f(&mut self.inner);
        self
    }
}

impl View for TestTriangleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut Context) {
        cx.assign_position(self.id)
    }

    fn properties(&self) -> &Properties { &self.inner }
}

impl IntoView for TestTriangleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}

pub struct TestCircleWidget {
    id: NodeId,
    inner: Properties,
}

impl TestCircleWidget {
    pub fn new() -> Self {
        let id = NodeId::new();
        let inner = Properties::new(Rgba::RED, (300, 300), Shape::Circle, false);
        Self { id, inner }
    }

    pub fn style<F: FnMut(&mut Properties)>(mut self, mut f: F) -> Self {
        f(&mut self.inner);
        self
    }
}

impl View for TestCircleWidget {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[Box<dyn View>]> { None }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut Context) {
        cx.assign_position(self.id)
    }

    fn properties(&self) -> &Properties { &self.inner }
}

impl IntoView for TestCircleWidget {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
