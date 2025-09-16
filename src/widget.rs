use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::{Shape, Scene};
use aplite_types::{Rgba, CornerRadius, Size, Rect};
use aplite_storage::{EntityManager, Entity, create_entity};

use crate::layout::*;
use crate::view::IntoView;
use crate::state::{
    ViewNode,
    NodeRef,
    AspectRatio,
};

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

thread_local! {
    pub(crate) static ENTITY_MANAGER: RefCell<EntityManager<WidgetId>> =
        RefCell::new(EntityManager::default());
}

create_entity! {
    pub WidgetId
}

/// main building block to create a renderable component
pub trait Widget {
    fn node(&self) -> ViewNode;

    fn node_ref(&self) -> Option<NodeRef> {
        self.node().node_ref()
    }

    fn id(&self) -> WidgetId {
        self.node().id()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        None
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        None
    }

    fn draw(&self, scene: &mut Scene) {
        let node = self.node_ref().unwrap().upgrade();

        if !node.borrow().flag.is_hidden() {
            if node.borrow().flag.is_dirty() {
                let state = node.borrow();

                scene.draw(&aplite_renderer::DrawArgs {
                    rect: &state.rect,
                    transform: &state.transform,
                    background_paint: &state.background_paint.as_paint_ref(),
                    border_paint: &state.border_paint.as_paint_ref(),
                    border_width: state.border_width.max(5.0),
                    shape: state.shape,
                    corner_radius: state.corner_radius,
                });

                drop(state);

                node.borrow_mut().flag.set_dirty(false);
            } else {
                scene.next_draw();
            }

            if let Some(children) = self.children_ref() {
                children
                    .iter()
                    .for_each(|child| {
                        child.draw(scene);
                    });
            }
        }
    }

    fn layout(&self, cx: &mut LayoutCx) -> bool {
        let node = self.node_ref().unwrap().upgrade();
        if node.borrow().flag.is_hidden() { return false }

        let size = node.borrow().rect.size();
        let mut this = node.borrow_mut();

        match cx.rules.orientation {
            Orientation::Vertical => {
                match cx.rules.align_h {
                    AlignH::Left | AlignH::Right => this.rect.x = cx.next_pos.x,
                    AlignH::Center => this.rect.x = cx.next_pos.x - size.width / 2.,
                }

                this.rect.y = cx.next_pos.y;
                cx.next_pos.y += cx.rules.spacing as f32 + size.height;
            },
            Orientation::Horizontal => {
                match cx.rules.align_v {
                    AlignV::Top | AlignV::Bottom => this.rect.y = cx.next_pos.y,
                    AlignV::Middle => this.rect.y = cx.next_pos.y - size.height / 2.,
                }

                this.rect.x = cx.next_pos.x;
                cx.next_pos.x += cx.rules.spacing as f32 + size.width;
            },
        }

        this.flag.set_dirty(true);

        true
    }

}

pub struct ChildrenRef<'a>(&'a Vec<Box<dyn Widget>>);

impl<'a> ChildrenRef<'a> {
    pub fn all_ref(&self) -> impl Iterator<Item = &'a dyn Widget> {
        self.0.iter().map(|child| child.as_ref())
    }

    pub fn all_boxed(&self) -> impl Iterator<Item = &'a Box<dyn Widget>> {
        self.0.iter()
    }

    pub fn visible_ref(&self) -> impl Iterator<Item = &'a dyn Widget> {
        self.0.iter()
            .filter_map(|child| {
                child.node()
                    .is_visible()
                    .then_some(child.as_ref())
            })
    }

    pub fn visible_boxed(&self) -> impl Iterator<Item = &'a Box<dyn Widget>> {
        self.0.iter()
            .filter_map(|child| {
                child.node()
                    .is_visible()
                    .then_some(child)
            })
    }
}

impl<'a> From<&'a Vec<Box<dyn Widget>>> for ChildrenRef<'a> {
    fn from(value: &'a Vec<Box<dyn Widget>>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ChildrenRef<'_> {
    type Target = Vec<Box<dyn Widget>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::fmt::Debug for ChildrenRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(self.0)
            .finish()
    }
}

pub struct ChildrenMut<'a>(&'a mut Vec<Box<dyn Widget>>);

impl<'a> From<&'a mut Vec<Box<dyn Widget>>> for ChildrenMut<'a> {
    fn from(value: &'a mut Vec<Box<dyn Widget>>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ChildrenMut<'_> {
    type Target = Vec<Box<dyn Widget>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::ops::DerefMut for ChildrenMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl std::fmt::Debug for ChildrenMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(self.0)
            .finish()
    }
}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn child(mut self, child: impl IntoView + 'static) -> Self {
        if let Some(children) = self.children_mut() {
            children.0.push(Box::new(child));
        }
        self
    }

    fn on<F>(self, event: WidgetEvent, f: F) -> Self
    where
        F: FnMut() + 'static,
    {
        CALLBACKS.with(|cell| {
            let mut storage = cell.borrow_mut();
            let callbacks = storage.entry(self.id()).or_default();
            callbacks.insert(event, Box::new(f));
        });

        self
    }

    fn image_aspect_ratio(self, val: AspectRatio) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .image_aspect_ratio = val;

        self
    }

    fn color(self, color: Rgba) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .background_paint = color.into();

        self
    }

    fn border_color(self, color: Rgba) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .border_paint = color.into();

        self
    }

    fn hover_color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn click_color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn border_width(self, val: f32) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .border_width = val;

        self
    }

    fn corner_radius(self, corner_radius: CornerRadius) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .corner_radius = corner_radius;

        self
    }

    fn shape(self, shape: Shape) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .shape = shape;

        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .rect.set_size(size.into());

        self
    }

    fn dragable(self) -> Self {
        let node = self.node_ref().unwrap().upgrade();
        let mut node = node.borrow_mut();
        node.flag.set_dragable(true);
        node.flag.set_hoverable(true);

        self
    }

    fn spacing(self, val: u8) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .spacing = val;

        self
    }

    fn padding(self, padding: Padding) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .padding = padding;

        self
    }

    fn min_width(self, val: f32) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .min_width = Some(val);

        self
    }

    fn min_height(self, val: f32) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .min_height = Some(val);

        self
    }

    fn max_width(self, val: f32) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .max_width = Some(val);

        self
    }

    fn max_height(self, val: f32) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .max_height = Some(val);

        self
    }

    fn align_h(self, align_h: AlignH) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .align_h = align_h;
        self
    }

    fn align_v(self, align_v: AlignV) -> Self {
        self.node_ref()
            .unwrap()
            .upgrade()
            .borrow_mut()
            .align_v = align_v;
        self
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

impl Widget for Box<dyn Widget> {
    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        self.as_mut().children_mut()
    }
}

impl Widget for Box<&mut dyn Widget> {
    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        self.as_ref().children_ref()
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        self.as_mut().children_mut()
    }
}

impl std::fmt::Debug for Box<dyn Widget> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("id", &self.id())
            .field("children", &self.children_ref().unwrap_or(ChildrenRef::from(&vec![])))
            .finish()
    }
}

// -------------------------------------

thread_local! {
    pub(crate) static CALLBACKS: RefCell<HashMap<WidgetId, CallbackStore>>
        = RefCell::new(Default::default());
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetEvent {
    Hover,
    LeftClick,
    RightClick,
    Drag,
    Input,
}

#[derive(Default)]
pub(crate) struct CallbackStore(Box<[Option<Box<dyn FnMut()>>; 5]>);

impl CallbackStore {
    pub(crate) fn insert(
        &mut self,
        event: WidgetEvent,
        callback: Box<dyn FnMut()>,
    ) {
        self.0[event as usize].replace(callback);
    }

    pub(crate) fn get_mut(&mut self, event: WidgetEvent) -> Option<&mut Box<dyn FnMut()>> {
        self.0[event as usize].as_mut()
    }
}

// -------------------------------------

pub(crate) struct WindowWidget {
    node: ViewNode,
    children: Vec<Box<dyn Widget>>,
}

impl WindowWidget {
    pub(crate) fn new(rect: Rect) -> Self {
        Self {
            node: ViewNode::window(rect),
            children: Vec::new(),
        }
    }
}

impl Widget for WindowWidget {
    fn node(&self) -> ViewNode {
        self.node
    }

    fn children_ref(&self) -> Option<ChildrenRef<'_>> {
        Some((&self.children).into())
    }

    fn children_mut(&mut self) -> Option<ChildrenMut<'_>> {
        Some((&mut self.children).into())
    }
}

// -------------------------------------

pub struct CircleWidget {
    node: ViewNode,
}

impl CircleWidget {
    pub fn new() -> Self {
        Self {
            node: ViewNode::default()
                .with_stroke_width(5.)
                .with_shape(Shape::Circle)
                .with_size((100., 100.)),
        }
    }
}

impl Widget for CircleWidget {
    fn node(&self) -> ViewNode {
        self.node
    }
}

// -------------------------------------

#[cfg(test)]
mod alt_widget {
    use aplite_types::*;
    use aplite_renderer::Shape;
    use aplite_storage::{Table, Tree, EntityManager, Entity, Component};
    use super::WidgetId;

    pub struct Context {
        entities: EntityManager<WidgetId>,
        table: Table<WidgetId>,
        tree: Tree<WidgetId>,
        current: Option<WidgetId>,
    }

    impl Context {
        fn new() -> Self {
            Self {
                entities: EntityManager::default(),
                table: Table::default(),
                tree: Tree::default(),
                current: None,
            }
        }

        fn create_entity(&mut self) -> WidgetId {
            let id = self.entities.create();
            if let Some(parent) = self.current.as_ref() {
                self.tree.insert(id, parent);
            } else {
                self.tree.insert_as_parent(id);
            }
            id
        }
    }

    pub trait WidgetTrait: Sized {
        fn id(&self) -> &WidgetId;

        fn child<F>(self, cx: &mut Context, child: F) -> Self
        where
            F: FnOnce(&mut Context)
        {
            let current = cx.current.replace(*self.id());
            child(cx);
            cx.current = current;
            self
        }

        fn component<C: Component>(self, cx: &mut Context, component: C) -> Self {
            cx.table.insert(self.id(), component);
            self
        }
    }

    pub struct TestWidget {
        id: WidgetId,
    }

    impl TestWidget {
        pub fn new(cx: &mut Context) -> Self {
            Self {
                id: cx.create_entity()
            }
        }
    }

    impl WidgetTrait for TestWidget {
        fn id(&self) -> &WidgetId {
            &self.id
        }
    }

    fn child1(cx: &mut Context) {
        TestWidget::new(cx)
            .component(cx, (Size::new(200., 100.), Shape::RoundedRect, Rgba::GREEN))
            .child(cx, |cx| {
                TestWidget::new(cx);
                TestWidget::new(cx);
                TestWidget::new(cx);
            });
    }

    fn child2(cx: &mut Context) {
        TestWidget::new(cx)
            .component(cx, (Size::new(200., 100.), Shape::RoundedRect, Rgba::BLUE));
    }

    fn app(cx: &mut Context) {
        TestWidget::new(cx)
            .component(cx, (Size::new(200., 100.), Shape::RoundedRect, Rgba::RED))
            .child(cx, child1)
            .child(cx, child2);
    }

    #[test]
    fn widget_ecs() {
        let mut cx = Context::new();
        app(&mut cx);
        assert_eq!(cx.tree.len(&WidgetId::root()), 7);
        assert_eq!(cx.tree.tree_depth(), 4);

        let query = cx.table.query::<(&Shape, &Size)>();
        let query2 = cx.table.query_one::<Rgba>();

        assert_eq!(query.count(), query2.count());
    }
}
