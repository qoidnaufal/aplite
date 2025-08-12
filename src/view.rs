use std::cell::RefCell;
use std::rc::{Rc, Weak};

use aplite_renderer::Shape;
use aplite_reactive::*;
use aplite_storage::{Tree, Entity, entity};
use aplite_types::{
    CornerRadius,
    Rgba,
    Paint,
    Size,
};

use crate::widget::Widget;
use crate::state::WidgetState;
use crate::context::layout::{
    Orientation,
    AlignH,
    AlignV,
};

entity! { pub ViewId }

thread_local! {
    pub(crate) static VIEW_STORAGE: ViewStorage = ViewStorage::new();
}

pub(crate) struct ViewStorage {
    // FIXME: this is nasty and slow, also i want the tree to store Box<dyn Widget>
    pub(crate) tree: RefCell<Tree<ViewId, Rc<RefCell<WidgetState>>>>,
    pub(crate) views: RefCell<Vec<View>>,
    pub(crate) hoverable: RefCell<Vec<ViewId>>,
    pub(crate) dirty: Signal<bool>,
}

impl ViewStorage {
    pub(crate) fn new() -> Self {
        Self {
            tree: RefCell::new(Tree::with_capacity(1024)),
            views: RefCell::new(Vec::with_capacity(1024)),
            hoverable: RefCell::new(Vec::new()),
            dirty: Signal::new(false),
        }
    }

    pub(crate) fn insert(&self, state: Rc<RefCell<WidgetState>>) -> ViewId {
        self.tree.borrow_mut().insert(state)
    }

    pub(crate) fn get_widget_state(&self, id: &ViewId) -> Option<Rc<RefCell<WidgetState>>> {
        self.tree.borrow().get(id).map(|rc| Rc::clone(rc))
    }

    pub(crate) fn append_child(&self, id: &ViewId, child: impl IntoView) {
        let child_id = child.id();
        let child_view = child.into_view();
        self.views.borrow_mut().push(child_view);
        self.tree.borrow_mut().add_child(id, child_id);
    }

    pub(crate) fn add_sibling(&self, id: &ViewId, sibling: impl IntoView) {
        let sibling_id = sibling.id();
        let sibling_view = sibling.into_view();
        self.views.borrow_mut().push(sibling_view);
        self.tree.borrow_mut().add_sibling(id, sibling_id);
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

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    inner: Box<dyn Widget>
}

impl View {
    fn new(widget: impl IntoView + 'static) -> Self {
        Self {
            inner: Box::new(widget),
        }
    }
}

impl Widget for View {
    fn node(&self) -> ViewNode {
        self.inner.node()
    }
}

impl Widget for Box<dyn IntoView> {
    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }
}

#[derive(Clone, Debug)]
pub struct ViewNode(
    pub(crate) ViewId,
    pub(crate) Weak<RefCell<WidgetState>>,
    pub(crate) SignalWrite<bool>,
);

impl ViewNode {
    pub fn new() -> Self {
        VIEW_STORAGE.with(|s| {
            let state = Rc::new(RefCell::new(WidgetState::new()));
            let inner = Rc::downgrade(&state);
            let id = s.insert(state);

            Self(id, inner, s.dirty.write_only())
        })
    }

    pub(crate) fn window(size: Size) -> Self {
        VIEW_STORAGE.with(|s| {
            let state = Rc::new(RefCell::new(WidgetState::window(size)));
            let inner = Rc::downgrade(&state);
            let id = s.insert(state);

            Self(id, inner, s.dirty.write_only())
        })
    }

    #[inline(always)]
    pub(crate) fn upgrade(&self) -> Option<Rc<RefCell<WidgetState>>> {
        self.1.upgrade()
    }

    pub fn with_name(self, name: &'static str) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().set_name(name);
        }
        self
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, size: impl Into<Size>) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().set_size(size);
        }
        self
    }

    pub fn with_min_width(self, val: f32) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().min_width = Some(val);
        }
        self
    }

    pub fn with_max_width(self, val: f32) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().max_width = Some(val);
        }
        self
    }

    pub fn with_min_height(self, val: f32) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().min_height = Some(val);
        }
        self
    }

    pub fn with_max_height(self, val: f32) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().max_height = Some(val);
        }
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(self, paint: impl Into<Paint>) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().background_paint = paint.into();
        }
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(self, color: impl Into<Paint>) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().border_paint = color.into();
        }
        self
    }

    pub fn with_stroke_width(self, val: f32) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().border_width = val;
        }
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().shape = shape;
        }
        self
    }

    pub fn with_rotation_deg(self, deg: f32) -> Self {
        self.with_rotation_rad(deg.to_radians())
    }

    pub fn with_rotation_rad(self, rad: f32) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().set_rotation_rad(rad);
        }
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().set_corner_radius(val);
        }
        self
    }

    pub fn with_horizontal_align(self, align_h: AlignH) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().align_h = align_h;
        }
        self
    }

    pub fn with_vertical_align(self, align_v: AlignV) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().align_v = align_v;
        }
        self
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().orientation = orientation;
        }
        self
    }

    pub fn hoverable(self) -> Self {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().hoverable = true;
        }
        VIEW_STORAGE.with(|s| {
            let mut hoverable = s.hoverable.borrow_mut();
            if !hoverable.contains(&self.0) {
                hoverable.push(self.0);
            }
        });
        self
    }

    pub fn set_color(&self, color: Rgba<u8>) {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().background_paint = color.into();
            self.2.set(true);
        }
    }

    pub fn set_shape(&self, shape: Shape) {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().shape = shape;
            self.2.set(true);
        }
    }

    pub fn set_rotation_deg(&self, deg: f32) {
        self.set_rotation_rad(deg.to_radians());
    }

    pub fn set_rotation_rad(&self, rad: f32) {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().rotation = rad;
            self.2.set(true);
        }
    }

    pub fn set_spacing(&self, val: f32) {
        if let Some(state) = self.upgrade() {
            state.borrow_mut().spacing = val;
            self.2.set(true);
        }
    }
}
