use std::collections::HashMap;
use std::cell::RefCell;
use aplite_reactive::*;
use aplite_types::{Matrix3x2, Rgba, Size, CornerRadius};
use aplite_renderer::{Element, ImageData, Shape};
use aplite_storage::{entity, Entity, Tree};

use crate::widget_state::WidgetState;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

entity! { pub ViewId }

// FIXME: this is kinda cheating, and not fun at all
thread_local! {
    pub(crate) static VIEW_STORAGE: ViewStorage = ViewStorage::new();
}

pub(crate) struct ViewStorage {
    pub(crate) tree: RefCell<Tree<ViewId>>,
    pub(crate) storage: RefCell<HashMap<ViewId, View>>,
    pub(crate) image_fn: RefCell<HashMap<ViewId, Box<dyn Fn() -> ImageData>>>,
    pub(crate) hoverable: RefCell<Vec<ViewId>>,
    pub(crate) dirty: RwSignal<Option<ViewId>>,
}

impl ViewStorage {
    fn new() -> Self {
        Self {
            tree: RefCell::new(Tree::with_capacity(1024)),
            storage: RefCell::new(HashMap::new()),
            image_fn: RefCell::new(HashMap::new()),
            hoverable: RefCell::new(Vec::new()),
            dirty: RwSignal::new(None),
        }
    }

    pub(crate) fn create_entity(&self) -> ViewId {
        self.tree.borrow_mut().create_entity()
    }

    // FIXME: there's logic error when appending on a fn() -> impl IntoView
    pub(crate) fn append_child(&self, id: &ViewId, child: impl IntoView) {
        let child_id = child.id();
        let state = child.widget_state();
        let child_root = state.root_id;

        if state.hoverable.get_untracked() || state.dragable.get_untracked() {
            let mut hoverable = self.hoverable.borrow_mut();
            if !hoverable.contains(&child_id) {
                hoverable.push(child_id);
            }
        }

        self.tree.borrow_mut().add_child(id, child_id);
        self.storage.borrow_mut().insert(child_id, child.into_view());

        let root = self.tree
            .borrow()
            .get_root(id)
            .copied()
            .unwrap_or(*id);
        child_root.set(Some(root));
    }

    pub(crate) fn add_sibling(&self, id: &ViewId, sibling: impl IntoView) {
        let sibling_id = sibling.id();
        let state = sibling.widget_state();
        let sibling_root = state.root_id;

        if state.hoverable.get_untracked() || state.dragable.get_untracked() {
            let mut hoverable = self.hoverable.borrow_mut();
            if !hoverable.contains(&sibling_id) {
                hoverable.push(sibling_id);
            }
        }

        self.tree.borrow_mut().add_sibling(id, sibling_id);
        self.storage.borrow_mut().insert(sibling_id, sibling.into_view());

        let root = self.tree
            .borrow()
            .get_root(id)
            .copied()
            .unwrap_or(*id);
        sibling_root.set(Some(root));
    }

    pub(crate) fn get_widget_state(&self, id: &ViewId) -> WidgetState {
        let storage = self.storage.borrow();
        storage[id].widget_state
    }

    pub(crate) fn get_all_members_of(&self, root_id: &ViewId) -> Vec<ViewId> {
        self.tree.borrow().get_all_members_of(root_id)
    }

    pub(crate) fn get_render_components(
        &self,
        root_id: &ViewId,
        screen: Size,
    ) -> Vec<(RwSignal<Element>, Matrix3x2, Option<Box<dyn Fn() -> ImageData>>)> {
        self.get_all_members_of(root_id)
            .iter()
            .enumerate()
            .filter_map(|(idx, view_id)| {
                self.storage
                    .borrow()
                    .get(view_id)
                    .map(|view| {
                        view.node.0.update(|elem| elem.set_transform_id(idx as _));
                        let image = self.image_fn.borrow_mut().remove(view_id);
                        let element = view.node.0;
                        let transform = view.widget_state.get_transform(screen);
                        (element, transform, image)
                    })
            })
            .collect()
    }
}

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

impl<T: Widget + 'static> IntoView for T {
    fn into_view(self) -> View {
        View::new(self)
    }
}

impl Widget for Box<dyn IntoView> {
    fn id(&self) -> ViewId {
        self.as_ref().id()
    }

    fn widget_state(&self) -> &WidgetState {
        self.as_ref().widget_state()
    }

    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    node: ViewNode,
    widget_state: WidgetState,
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            node: widget.node(),
            widget_state: *widget.widget_state(),
        }
    }

    pub(crate) fn window(size: Size) -> Self {
        let window_state = WidgetState::window(size);
        Self {
            node: ViewNode::new(),
            widget_state: window_state,
        }
    }

    pub(crate) fn widget_state(&self) -> WidgetState {
        self.widget_state
    }
}

impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.widget_state();
        let name = state.name;

        f.debug_struct("View")
            .field("name", &name)
            .finish()
    }
}

/// main building block to create a renderable component
pub trait Widget {
    fn id(&self) -> ViewId;
    fn widget_state(&self) -> &WidgetState;
    fn node(&self) -> ViewNode;
}

pub struct CircleWidget {
    id: ViewId,
    node: ViewNode,
    state: WidgetState,
}

impl CircleWidget {
    pub fn new() -> Self {
        let id = VIEW_STORAGE.with(|s| s.create_entity());
        let node = ViewNode::new()
            .with_shape(Shape::Circle)
            .with_stroke_width(5);
        let state = WidgetState::new()
            .with_name("Circle")
            .with_size((100, 100));

        Self {
            id,
            node,
            state,
        }
    }
    
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        let trigger = self.state.trigger_callback;
        Effect::new(move |_| {
            if trigger.get() {
                f();
                trigger.set_untracked(false);
            }
        });
        self
    }

    pub fn state(mut self, f: impl Fn(&mut WidgetState)) -> Self {
        f(&mut self.state);
        self
    }
}

impl Widget for CircleWidget {
    fn id(&self) -> ViewId {
        self.id
    }

    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn node(&self) -> ViewNode {
        self.node
    }
}

/// A wrapper over [`Element`]
#[derive(Clone, Copy)]
pub struct ViewNode(RwSignal<Element>);

impl ViewNode {
    pub fn new() -> Self {
        Self(RwSignal::new(Element::new()))
    }

    pub fn with_fill_color(self, color: Rgba<u8>) -> Self {
        self.set_fill_color(color);
        self
    }

    pub fn with_stroke_color(self, color: Rgba<u8>) -> Self {
        self.set_stroke_color(color);
        self
    }

    pub fn with_stroke_width(self, val: u32) -> Self {
        self.set_stroke_width(val);
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        self.set_shape(shape);
        self
    }

    pub fn with_rotation(self, val: f32) -> Self {
        self.set_rotation(val);
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        self.set_corner_radius(val);
        self
    }

    pub(crate) fn set_fill_color(&self, color: Rgba<u8>) {
        self.0.update_untracked(|el| el.set_fill_color(color));
    }

    pub(crate) fn set_stroke_color(&self, color: Rgba<u8>) {
        self.0.update_untracked(|el| el.set_stroke_color(color));
    }

    pub(crate) fn set_stroke_width(&self, val: u32) {
        self.0.update_untracked(|el| el.set_stroke_width(val));
    }

    pub(crate) fn set_shape(&self, shape: Shape) {
        self.0.update_untracked(|el| el.set_shape(shape));
    }

    /// value must be in degree
    pub(crate) fn set_rotation(&self, val: f32) {
        self.0.update_untracked(|el| el.set_rotation(val.to_radians()));
    }

    pub(crate) fn set_corner_radius(&self, val: CornerRadius) {
        self.0.update_untracked(|el| el.set_corner_radius(val));
    }
}

/// this is just a wrapper over `FnMut(Option<T>) -> T`
pub trait FnEl<T>: FnMut(Option<T>) -> T {}

impl<F, T> FnEl<T> for F where F: FnMut(Option<T>) -> T {}

/// trait to modify the rendered element
pub trait Style: Widget + Sized {
    fn set_color<F>(self, mut f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static,
    {
        let node = self.node();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let color = f(prev);
            node.set_fill_color(color);
            dirty.set(root_id.get_untracked());
            color
        });
        self
    }

    fn set_stroke_color<F>(self, mut f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static
    {
        let node = self.node();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let color = f(prev);
            node.set_stroke_color(color);
            dirty.set(root_id.get_untracked());
            color
        });
        self
    }

    fn set_hover_color<F>(self, mut f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static
    {
        self.widget_state().hoverable.set_untracked(true);

        let node = self.node();
        let init_color = node.0.read_untracked(|elem| elem.fill_color());
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let is_hovered = self.widget_state().is_hovered;
        let is_clicked = self.widget_state().is_clicked;
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let color = f(prev);
            if is_hovered.get() {
                if !is_clicked.get() {
                    node.set_fill_color(color);
                }
            } else {
                node.set_fill_color(init_color);
            }
            dirty.set(root_id.get_untracked());
            color
        });
        self
    }

    fn set_click_color<F>(self, mut f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static,
    {
        self.widget_state().hoverable.set_untracked(true);
        let node = self.node();
        let is_clicked = self.widget_state().is_clicked;
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let color = f(prev);
            if is_clicked.get() {
                node.set_fill_color(color);
            }
            dirty.set(root_id.get_untracked());
            color
        });
        self
    }

    fn set_stroke_width<F>(self, mut f: F) -> Self
    where
        F: FnEl<u32> + 'static
    {
        let node = self.node();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let val = f(prev);
            node.set_stroke_width(val);
            dirty.set(root_id.get_untracked());
            val
        });
        self
    }

    fn set_rotation<F>(self, mut f: F) -> Self
    where
        F: FnEl<f32> + 'static
    {
        let node = self.node();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let val = f(prev);
            node.set_rotation(val);
            dirty.set(root_id.get_untracked());
            val
        });
        self
    }

    fn set_corners<F>(self, mut f: F) -> Self
    where
        F: FnEl<CornerRadius> + 'static
    {
        let node = self.node();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let val = f(prev);
            node.set_corner_radius(val);
            dirty.set(root_id.get_untracked());
            val
        });
        self
    }

    fn set_shape<F>(self, mut f: F) -> Self
    where
        F: FnEl<Shape> + 'static
    {
        let node = self.node();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let shape = f(prev);
            node.set_shape(shape);
            dirty.set(root_id.get_untracked());
            shape
        });
        self
    }

    fn set_size(self, size: impl Into<Size>) -> Self {
        self.widget_state()
            .rect
            .update_untracked(|rect| rect.set_size(size.into()));
        self
    }

    fn set_dragable(self, value: bool) -> Self {
        self.widget_state()
            .dragable
            .update_untracked(|val| *val = value);
        self
    }
}

impl<T> Style for T where T: Widget + Sized {}

// TODO: is immediately calculate the size here a good idea?
pub trait Layout: Widget + Sized {
    fn child(self, child: impl IntoView) -> Self {
        let self_z_index = self.widget_state().z_index;
        let child_z_index = child.widget_state().z_index;

        let self_root = self.widget_state().root_id;
        let child_root = child.widget_state().root_id;

        Effect::new(move |_| {
            child_z_index.set(self_z_index.get() + 1);
            child_root.set(self_root.get());
        });

        VIEW_STORAGE.with(|s| s.append_child(&self.id(), child));
        self
    }

    fn and(self, sibling: impl IntoView) -> Self {
        let self_z_index = self.widget_state().z_index;
        let sibling_z_index = sibling.widget_state().z_index;

        let self_root = self.widget_state().root_id;
        let sibling_root = sibling.widget_state().root_id;

        Effect::new(move |_| {
            sibling_z_index.set(self_z_index.get());
            sibling_root.set(self_root.get());
        });

        VIEW_STORAGE.with(|s| s.add_sibling(&self.id(), sibling));
        self
    }
}

impl<T> Layout for T where T: Widget + Sized {}

#[allow(unused)]
mod alt_view {
    use std::collections::HashMap;
    use aplite_storage::{Tree, Entity, entity};

    entity! { ViewIdAlt, }

    struct ContextAlt {
        tree: Tree<ViewIdAlt>,
        comp1: HashMap<ViewIdAlt, Comp1>,
        comp2: HashMap<ViewIdAlt, Comp2>,
        comp3: HashMap<ViewIdAlt, Comp3>,
    }

    struct Comp1 {}
    struct Comp2 {}
    struct Comp3 {}

    struct ButtonAlt {
        id: ViewIdAlt,
    }

    impl ButtonAlt {
        fn new(cx: &mut ContextAlt) -> Self {
            let id = cx.tree.create_entity();
            let comp1 = Comp1 {};
            cx.comp1.insert(id, comp1);
            Self { id, }
        }
    }

    trait WidgetAlt: Sized {
        fn id(&self) -> ViewIdAlt;

        fn append_child(self, cx: &mut ContextAlt, child: impl IntoViewAlt) -> Self {
            cx.tree.add_child(&self.id(), child.id());
            self
        }
    }

    impl WidgetAlt for ButtonAlt {
        fn id(&self) -> ViewIdAlt {
            self.id
        }
    }

    struct ViewAlt {
        id: ViewIdAlt,
    }

    trait IntoViewAlt: WidgetAlt {
        fn into_view(self) -> ViewAlt {
            ViewAlt { id: self.id() }
        }
    }

    impl<T: WidgetAlt> IntoViewAlt for T {}

    #[cfg(test)]
    mod alt_test {
        use super::*;

        fn root(cx: &mut ContextAlt) -> impl IntoViewAlt {
            let first = ButtonAlt::new(cx);
            let parent = ButtonAlt::new(cx).append_child(cx, first);

            branch(cx, parent)
        }

        fn branch(cx: &mut ContextAlt, parent: impl IntoViewAlt) -> impl IntoViewAlt {
            // if let Some(first_child) = cx.tree.get_first_child(&parent.id()).copied() {
            //     let second = ButtonAlt::new(cx);
            //     cx.tree.add_child(&first_child, second.id());
            // }
            let second = ButtonAlt::new(cx);
            parent.append_child(cx, second)
        }

        #[test]
        fn alt() {
            let mut cx = ContextAlt {
                tree: Tree::new(),
                comp1: HashMap::new(),
                comp2: HashMap::new(),
                comp3: HashMap::new(),
            };

            let root = root(&mut cx).into_view();
            let child = *cx.tree.get_first_child(&root.id).unwrap();

            eprintln!("{:?}", cx.tree);
            assert_eq!(child, ViewIdAlt(0));
        }
    }
}
