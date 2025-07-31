use std::cell::{RefCell, Ref, RefMut};
use std::rc::{Rc, Weak};

use aplite_reactive::*;
use aplite_types::CornerRadius;
use aplite_types::{Rgba, Paint};
use aplite_renderer::{Element, Shape};
use aplite_storage::{entity, Entity, Tree, Map, IndexMap};

use crate::widget::Widget;
use crate::widget_state::WidgetState;

entity! { pub ViewId, pub PaintId }

// FIXME: this is kinda cheating, and not fun at all
thread_local! {
    pub(crate) static VIEW_STORAGE: ViewStorage = ViewStorage::new();
}

pub(crate) struct ViewStorage {
    pub(crate) tree: RefCell<Tree<ViewId, WidgetState>>,
    pub(crate) storage: RefCell<Map<ViewId, View>>,
    // WARN: do you really need separate id for paint?
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

    pub(crate) fn insert(&self, data: WidgetState) -> ViewId {
        self.tree.borrow_mut().insert(data)
    }

    pub(crate) fn add_paint(&self, paint: Paint) -> PaintId {
        self.paint.borrow_mut().insert_no_duplicate(paint)
    }

    // FIXME: there's logic error when appending on a fn() -> impl IntoView
    pub(crate) fn append_child(&self, id: &ViewId, child: impl IntoView) {
        let child_id = child.id();
        let tree = self.tree.borrow();
        let state = tree.get(&child_id).unwrap();
        let child_root = state.root_id;

        drop(tree);

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
        let tree = self.tree.borrow();
        let state = tree.get(&sibling_id).unwrap();
        let sibling_root = state.root_id;

        drop(tree);

        self.tree.borrow_mut().add_sibling(id, sibling_id);
        self.storage.borrow_mut().insert(sibling_id, sibling.into_view());

        let root = self.tree
            .borrow()
            .get_root(id)
            .copied()
            .unwrap_or(*id);
        sibling_root.set(Some(root));
    }

    #[inline(always)]
    pub(crate) fn get_all_members_of(&self, root_id: &ViewId) -> Vec<ViewId> {
        self.tree.borrow().get_all_members_of(root_id)
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
    pub(crate) node: ViewNode,
    pub(crate) paint_id: PaintId,
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            node: widget.node(),
            paint_id: widget.paint_id(),
        }
    }

    pub(crate) fn window() -> Self {
        let paint_id = VIEW_STORAGE.with(|s| s.paint
            .borrow_mut()
            .insert(Paint::Color(Rgba::TRANSPARENT))
        );
        Self {
            node: ViewNode::new(),
            paint_id,
        }
    }
}

// FIXME: shouldn't be needed
/// A wrapper over [`Element`]
pub struct ViewNode(pub(crate) Rc<RefCell<Element>>);

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

    #[allow(unused)]
    pub(crate) fn downgrade(&self) -> Weak<RefCell<Element>> {
        Rc::downgrade(&self.0)
    }

    pub(crate) fn borrow(&self) -> Ref<'_, Element> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<'_, Element> {
        self.0.borrow_mut()
    }
}
