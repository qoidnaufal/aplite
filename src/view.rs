use std::cell::RefCell;

use aplite_reactive::*;
use aplite_renderer::Shape;
use aplite_storage::{Tree, U64Map};
use aplite_types::{
    CornerRadius,
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

aplite_macro::entity! { pub ViewId }

// FIXME: this is kinda cheating, and not fun at all
thread_local! {
    pub(crate) static VIEW_STORAGE: ViewStorage = ViewStorage::new();
}

pub(crate) struct ViewStorage {
    pub(crate) tree: RefCell<Tree<ViewId, WidgetState>>,
    pub(crate) storage: RefCell<U64Map<ViewId, View>>,

    // WARN: do you really need separate id for paint?
    pub(crate) hoverable: RefCell<Vec<ViewId>>,
    pub(crate) dirty: Signal<bool>,
}

impl ViewStorage {
    fn new() -> Self {
        Self {
            tree: RefCell::new(Tree::with_capacity(1024)),
            storage: RefCell::new(U64Map::new()),
            hoverable: RefCell::new(Vec::new()),
            dirty: Signal::new(false),
        }
    }

    pub(crate) fn insert(&self, data: WidgetState) -> ViewId {
        self.tree.borrow_mut().insert(data)
    }

    // FIXME: there's logic error when appending on a fn() -> impl IntoView
    pub(crate) fn append_child(&self, id: &ViewId, child: impl IntoView) {
        let child_id = child.id();
        let tree = self.tree.borrow();

        drop(tree);

        self.tree.borrow_mut().add_child(id, child_id);
        self.storage.borrow_mut().insert(child_id, child.into_view());
    }

    pub(crate) fn add_sibling(&self, id: &ViewId, sibling: impl IntoView) {
        let sibling_id = sibling.id();
        let tree = self.tree.borrow();

        drop(tree);

        self.tree.borrow_mut().add_sibling(id, sibling_id);
        self.storage.borrow_mut().insert(sibling_id, sibling.into_view());
    }

    #[inline(always)]
    pub(crate) fn get_all_members_of(&self, root_id: &ViewId) -> Vec<ViewId> {
        self.tree.borrow().get_all_members_of(root_id)
    }
}

pub trait IntoView: Widget {
    fn into_view(self) -> View;
}

impl<T: Widget> IntoView for T {
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
}

/// wrapper over [`Widget`] trait to be stored inside [`ViewStorage`]
pub struct View {
    pub(crate) node: ViewNode,
}

impl View {
    fn new(widget: impl IntoView) -> Self {
        Self {
            node: widget.node(),
        }
    }

    pub(crate) fn window(size: Size) -> Self {
        Self {
            node: ViewNode::window(size),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ViewNode(pub(crate) ViewId);

impl ViewNode {
    pub fn new() -> Self {
        VIEW_STORAGE.with(|s| {
            let state = WidgetState::new();
            let id = s.insert(state);

            Self(id)
        })
    }

    pub(crate) fn window(size: Size) -> Self {
        VIEW_STORAGE.with(|s| {
            let state = WidgetState::window(size);
            let id = s.insert(state);

            Self(id)
        })
    }

    #[inline(always)]
    pub fn id(&self) -> ViewId {
        self.0
    }

    pub fn with_name(self, name: &'static str) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.set_name(name);
            }
        });
        self
    }

    /// Types which implement [`Into<Size>`] are:
    /// - (u32, u32)
    /// - (f32, f32)
    /// - [`Size`](aplite_types::Size)
    pub fn with_size(self, size: impl Into<Size>) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.set_size(size);
            }
        });
        self
    }

    pub fn with_min_width(self, val: f32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.min_width = Some(val);
            }
        });
        self
    }

    pub fn with_max_width(self, val: f32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.max_width = Some(val);
            }
        });
        self
    }

    pub fn with_min_height(self, val: f32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.min_height = Some(val);
            }
        });
        self
    }

    pub fn with_max_height(self, val: f32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.max_height = Some(val);
            }
        });
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_background_paint(self, paint: impl Into<Paint>) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.background = paint.into();
            }
        });
        self
    }

    /// Types which implement [`Into<Paint>`] are:
    /// - [`ImageData`](aplite_types::ImageData)
    /// - [`Rgba`](aplite_types::Rgba)
    pub fn with_border_paint(self, color: impl Into<Paint>) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.border_color = color.into();
            }
        });
        self
    }

    pub fn with_stroke_width(self, val: u32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.border_width = val as _;
            }
        });
        self
    }

    pub fn with_shape(self, shape: Shape) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.shape = shape;
            }
        });
        self
    }

    pub fn with_rotation_deg(self, deg: f32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.set_rotation_deg(deg);
            }
        });
        self
    }

    pub fn with_rotation_rad(self, rad: f32) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.set_rotation_rad(rad);
            }
        });
        self
    }

    pub fn with_corner_radius(self, val: CornerRadius) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.set_corner_radius(val);
            }
        });
        self
    }

    pub fn with_horizontal_align(self, align_h: AlignH) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.align_h = align_h;
            }
        });
        self
    }

    pub fn with_vertical_align(self, align_v: AlignV) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.align_v = align_v;
            }
        });
        self
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.orientation = orientation;
            }
        });
        self
    }

    pub fn set_hoverable(self) -> Self {
        VIEW_STORAGE.with(|s| {
            let mut tree = s.tree.borrow_mut();
            if let Some(state) = tree.get_mut(&self.0) {
                state.hoverable = true;
            }
            s.hoverable.borrow_mut().push(self.0);
        });
        self
    }
}
