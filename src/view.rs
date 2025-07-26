use std::cell::{RefCell, Ref, RefMut};
use std::rc::{Rc, Weak};

use aplite_reactive::*;
use aplite_types::{Size, CornerRadius};
use aplite_types::{Rgba, Paint};
use aplite_renderer::{Element, Shape, Renderer};
use aplite_storage::{entity, Entity, Tree, Map, IndexMap};

use crate::widget_state::WidgetState;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

entity! { pub ViewId, pub PaintId }

// FIXME: this is kinda cheating, and not fun at all
thread_local! {
    pub(crate) static VIEW_STORAGE: ViewStorage = ViewStorage::new();
}

pub(crate) struct ViewStorage {
    pub(crate) tree: RefCell<Tree<ViewId>>,
    pub(crate) storage: RefCell<Map<ViewId, View>>,
    pub(crate) paint: RefCell<IndexMap<PaintId, Paint>>,
    pub(crate) hoverable: RefCell<Vec<ViewId>>,
    pub(crate) dirty: Signal<Option<ViewId>>,
}

impl ViewStorage {
    fn new() -> Self {
        Self {
            tree: RefCell::new(Tree::with_capacity(1024)),
            storage: RefCell::new(Map::new()),
            paint: RefCell::new(IndexMap::new()),
            hoverable: RefCell::new(Vec::new()),
            dirty: Signal::new(None),
        }
    }

    pub(crate) fn create_entity(&self) -> ViewId {
        self.tree.borrow_mut().create_entity()
    }

    pub(crate) fn add_paint(&self, paint: Paint) -> PaintId {
        self.paint.borrow_mut().insert_no_duplicate(paint)
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

    #[inline(always)]
    pub(crate) fn get_all_members_of(&self, root_id: &ViewId) -> Vec<ViewId> {
        self.tree.borrow().get_all_members_of(root_id)
    }

    pub(crate) fn get_render_components(
        &self,
        root_id: &ViewId,
        renderer: &mut Renderer,
    ) {
        self.get_all_members_of(root_id)
            .iter()
            .enumerate()
            .for_each(|(idx, view_id)| {
                if let Some(view) = self.storage
                    .borrow()
                    .get(view_id) {
                        view.node.0.borrow_mut().set_transform_id(idx as _);

                        let paint_storage = self.paint.borrow();
                        let paint_ref = paint_storage
                            .get(&view.paint_id)
                            .map(|paint| paint.as_paint_ref())
                            .unwrap();

                        let element = view.node.clone();
                        let transform = view.widget_state.get_transform(renderer.screen_res());

                        renderer.paint(*element.borrow(), transform, paint_ref);
                    }
            })
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

    fn paint_id(&self) -> PaintId {
        self.as_ref().paint_id()
    }
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    // FIXME: this shouldn't be needed here
    node: ViewNode,
    widget_state: WidgetState,
    paint_id: PaintId,
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            node: widget.node(),
            widget_state: *widget.widget_state(),
            paint_id: widget.paint_id(),
        }
    }

    pub(crate) fn window(size: Size) -> Self {
        let window_state = WidgetState::window(size);
        let paint_id = VIEW_STORAGE.with(|s| s.paint
            .borrow_mut()
            .insert(Paint::Color(Rgba::TRANSPARENT))
        );
        Self {
            node: ViewNode::new(),
            widget_state: window_state,
            paint_id,
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
    fn paint_id(&self) -> PaintId;
}

pub struct CircleWidget {
    id: ViewId,
    node: ViewNode,
    state: WidgetState,
    paint_id: PaintId,
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

        let paint_id = VIEW_STORAGE
            .with(|s| s.paint
                .borrow_mut()
                .insert(Paint::Color(Rgba::RED))
            );

        Self {
            id,
            node,
            state,
            paint_id,
        }
    }
    
    pub fn on_click<F: Fn() + 'static>(self, f: F) -> Self {
        let trigger = self.state.trigger_callback;
        Effect::new(move |_| {
            if trigger.get() {
                f();
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
        self.node.clone()
    }

    fn paint_id(&self) -> PaintId {
        self.paint_id
    }
}

/// A wrapper over [`Element`]
pub struct ViewNode(Rc<RefCell<Element>>);

impl Clone for ViewNode {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl ViewNode {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Element::new())))
    }

    pub fn with_fill_color(self, color: Rgba<u8>) -> Self {
        self.0.borrow_mut().set_fill_color(color);
        self
    }

    pub fn with_stroke_color(self, color: Rgba<u8>) -> Self {
        self.0.borrow_mut().set_stroke_color(color);
        self
    }

    pub fn with_stroke_width(self, val: u32) -> Self {
        self.0.borrow_mut().set_stroke_width(val);
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        self.0.borrow_mut().set_shape(shape);
        self
    }

    pub fn with_rotation(self, val: f32) -> Self {
        self.0.borrow_mut().set_rotation(val);
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        self.0.borrow_mut().set_corner_radius(val);
        self
    }

    pub(crate) fn weak_ref(&self) -> Weak<RefCell<Element>> {
        Rc::downgrade(&self.0)
    }

    pub(crate) fn borrow(&self) -> Ref<'_, Element> {
        self.0.borrow()
    }

    #[allow(unused)]
    pub(crate) fn borrow_mut(&self) -> RefMut<'_, Element> {
        self.0.borrow_mut()
    }
}

/// this is just a wrapper over `FnMut(Option<T>) -> T`
pub trait FnEl<T>: FnMut(Option<T>) -> T {}

impl<F, T> FnEl<T> for F where F: FnMut(Option<T>) -> T {}

/// this is just a wrapper over `FnMut() -> T`
pub trait FnAction<T>: FnMut() -> T {}

impl<F, T> FnAction<T> for F where F: FnMut() -> T {}

// FIXME: there are too many effects here, maybe shouldn't
/// trait to modify the rendered element
pub trait Style: Widget + Sized {
    fn set_color<F>(self, mut f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static,
    {
        let node = self.node().weak_ref();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let color = f(prev);
            if let Some(node) = node.upgrade() {
                node.borrow_mut().set_fill_color(color);
                dirty.set(root_id.get_untracked());
            }
            color
        });
        self
    }

    fn set_stroke_color<F>(self, mut f: F) -> Self
    where
        F: FnEl<Rgba<u8>> + 'static
    {
        let node = self.node().weak_ref();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let color = f(prev);
            if let Some(node) = node.upgrade() {
                node.borrow_mut().set_stroke_color(color);
                dirty.set(root_id.get_untracked());
            }
            color
        });
        self
    }

    fn set_hover_color<F>(self, mut f: F) -> Self
    where
        F: FnAction<Rgba<u8>> + 'static
    {
        self.widget_state().hoverable.set_untracked(true);

        let node = self.node();
        let init_color = node.0.borrow().fill_color();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let is_hovered = self.widget_state().is_hovered;
        let is_clicked = self.widget_state().is_clicked;
        let root_id = self.widget_state().root_id;

        let weak_node = node.weak_ref();

        Effect::new(move |_| {
            if is_hovered.get() {
                if !is_clicked.get() {
                    let color = f();
                    if let Some(node) = weak_node.upgrade() {
                        node.borrow_mut().set_fill_color(color);
                        dirty.set(root_id.get_untracked());
                    }
                }
            } else {
                if let Some(node) = weak_node.upgrade() {
                    node.borrow_mut().set_fill_color(init_color);
                    dirty.set(root_id.get_untracked());
                }
            }
        });
        self
    }

    fn set_click_color<F>(self, mut f: F) -> Self
    where
        F: FnAction<Rgba<u8>> + 'static,
    {
        self.widget_state().hoverable.set_untracked(true);

        let node = self.node().weak_ref();
        let is_clicked = self.widget_state().is_clicked;
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |_| {
            if is_clicked.get() {
                let color = f();
                if let Some(node) = node.upgrade() {
                    node.borrow_mut().set_fill_color(color);
                    dirty.set(root_id.get_untracked());
                }
            }
        });
        self
    }

    fn set_stroke_width<F>(self, mut f: F) -> Self
    where
        F: FnEl<u32> + 'static
    {
        let node = self.node().weak_ref();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let val = f(prev);
            if let Some(node) = node.upgrade() {
                node.borrow_mut().set_stroke_width(val);
                dirty.set(root_id.get_untracked());
            }
            val
        });
        self
    }

    fn set_rotation<F>(self, mut f: F) -> Self
    where
        F: FnEl<f32> + 'static
    {
        let node = self.node().weak_ref();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let val = f(prev);
            if let Some(node) = node.upgrade() {
                node.borrow_mut().set_rotation(val.to_radians());
                dirty.set(root_id.get_untracked());
            }
            val
        });
        self
    }

    fn set_corners<F>(self, mut f: F) -> Self
    where
        F: FnEl<CornerRadius> + 'static
    {
        let node = self.node().weak_ref();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let val = f(prev);
            if let Some(node) = node.upgrade() {
                node.borrow_mut().set_corner_radius(val);
                dirty.set(root_id.get_untracked());
            }
            val
        });
        self
    }

    fn set_shape<F>(self, mut f: F) -> Self
    where
        F: FnEl<Shape> + 'static
    {
        let node = self.node().weak_ref();
        let dirty = VIEW_STORAGE.with(|s| s.dirty);
        let root_id = self.widget_state().root_id;

        Effect::new(move |prev| {
            let shape = f(prev);
            if let Some(node) = node.upgrade() {
                node.borrow_mut().set_shape(shape);
                dirty.set(root_id.get_untracked());
            }
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
