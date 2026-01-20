use std::collections::HashMap;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_storage::SlotId;
use aplite_types::{Rect, Size, Vec2f};

use crate::layout::{AlignH, AlignV, Axis, LayoutRules, Padding, Spacing};
use crate::cursor::{Cursor, EmittedClickEvent, MouseAction, MouseButton};
use crate::widget::{Renderable, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(pub(crate) u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewPath(pub(crate) Vec<u32>);

pub struct BuildCx<'a> {
    pub(crate) cx: &'a mut Context,
    pub(crate) id: u64,
}

pub struct LayoutCx<'a> {
    pub(crate) cx: &'a mut Context,
    pub(crate) bound: Rect,
    pub(crate) rules: LayoutRules,
}

pub struct CursorCx<'a> {
    pub(crate) cx: &'a mut Context,
}

pub struct Context {
    pub(crate) elements: Vec<Box<dyn Renderable>>,
    pub(crate) layout_nodes: Vec<Rect>,
    pub(crate) view_ids: HashMap<Box<[u32]>, ViewId>,
    pub(crate) view_path: ViewPath,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pub(crate) window_rect: Rect,
    pub(crate) pending_update: Vec<SlotId>,
}

impl Context {
    pub(crate) fn new(size: Size) -> Self {
        Self {
            elements: Vec::new(),
            layout_nodes: Vec::new(),
            view_ids: HashMap::new(),
            view_path: ViewPath::new(),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            window_rect: Rect::from_size(size),
            pending_update: Vec::new(),
        }
    }

    pub fn build<T: Widget>(&mut self, view: &T) {
        let mut cx = BuildCx::new(self);
        cx.with_id(0, |cx| view.build(cx));

        let len = cx.id as usize;
        self.elements.truncate(len);
    }
 
    pub fn layout<T: Widget>(&mut self, view: &T) {
        let rules = LayoutRules {
            padding: Padding::default(),
            axis: Axis::Vertical,
            align_h: AlignH::Left,
            align_v: AlignV::Top,
            spacing: Spacing(5),
        };

        let mut cx = LayoutCx::new(self, rules, self.window_rect);
        cx.with_id(0, |cx| view.layout(cx));

        let len = self.elements.len();
        self.layout_nodes.truncate(len);
    }

    pub(crate) fn toggle_dirty(&self) {
        self.dirty.set(true);
    }

    pub(crate) fn process_pending_update(&mut self) {
        if !self.pending_update.is_empty() {
            self.pending_update
                .drain(..)
                .for_each(|_id| {
                });
            self.toggle_dirty();
        }
    }

    pub(crate) fn handle_mouse_move<T: Widget>(&mut self, pos: impl Into<Vec2f>, view: &T) {
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        let mut cx = CursorCx::new(self);
        cx.with_id(0, |cx| view.detect_hover(cx));
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_drag();
    }

    pub(crate) fn handle_drag(&mut self) {}

    pub(crate) fn handle_click(
        &mut self,
        action: impl Into<MouseAction>,
        button: impl Into<MouseButton>
    ) {
        match self.cursor.process_click_event(action.into(), button.into()) {
            EmittedClickEvent::Captured(id) => {
                let node = &self.layout_nodes[id.0 as usize];
                let pos = node.vec2f();
                self.cursor.click.offset = self.cursor.click.pos - pos;
            },
            EmittedClickEvent::TriggerCallback(callback) => {
                unsafe {
                    let cb = &*callback;
                    cb()
                }
            },
            _ => {}
        }
    }

    pub(crate) fn render<W: Widget>(&self, _widget: &W, renderer: &mut Renderer) {
        let mut scene = renderer.scene();
        self.layout_nodes
            .iter()
            .zip(&self.elements)
            .for_each(|(rect, state)| state.render(rect, &mut scene));
    }
}

macro_rules! impl_context {
    ($cx:ident<$lifetime:lifetime>) => {
        impl<$lifetime> $cx<$lifetime> {
            pub(crate) fn pop(&mut self) -> u32 {
                self.cx.view_path.0.pop().unwrap_or_default()
            }

            pub(crate) fn push(&mut self, path_id: u32) {
                self.cx.view_path.0.push(path_id);
            }

            pub fn with_id<R: 'static>(&mut self, id_path: u32, f: impl FnOnce(&mut Self) -> R) -> R {
                self.push(id_path);
                let res = f(self);
                self.pop();
                res
            }

            pub fn get_id(&self) -> Option<&ViewId> {
                let path = self.get_path();
                self.cx.view_ids.get(&path)
            }

            pub fn get_parent_id(&self) -> Option<&ViewId> {
                if self.cx.view_path.0.is_empty() {
                    None
                } else {
                    let parent_path = self.cx.view_path
                        .0[..self.cx.view_path.0.len() - 1]
                        .iter()
                        .copied()
                        .collect::<Box<[_]>>();
                    self.cx.view_ids.get(&parent_path)
                }
            }

            pub fn get_path(&self) -> Box<[u32]> {
                self.cx.view_path.0.clone().into_boxed_slice()
            }

            pub fn get_z_index(&self) -> u32 {
                self.cx.view_path.0.len() as u32
            }

            pub fn get_element<S: 'static>(&self) -> Option<&S> {
                self.get_id()
                    .map(|id| unsafe {
                        let ptr: *const dyn Renderable = self.cx.elements[id.0 as usize].as_ref();
                        &*ptr.cast::<S>()
                    })
            }

            pub fn get_element_mut<S: 'static>(&mut self) -> Option<&mut S> {
                self.get_id()
                    .copied()
                    .map(|id| unsafe {
                        let ptr: *mut dyn Renderable = self.cx.elements[id.0 as usize].as_mut();
                        &mut *ptr.cast::<S>()
                    })
            }
        }
    };
}

impl_context!(BuildCx<'a>);
impl_context!(LayoutCx<'a>);
impl_context!(CursorCx<'a>);

/*
#########################################################
#
# impl BuildCx
#
#########################################################
*/

impl<'a> BuildCx<'a> {
    pub(crate) fn new(cx: &'a mut Context) -> Self {
        Self {
            cx,
            id: 0,
        }
    }

    pub fn register_element<R: Renderable + 'static>(&mut self, state: R) {
        let id = self.create_id();
        let path = self.get_path();
        self.cx.view_ids.insert(path, id);

        if let Some(any) = self.cx.elements.get_mut(id.0 as usize) {
            *any = Box::new(state);
        } else {
            self.cx.elements.push(Box::new(state));
        }
    }

    fn create_id(&mut self) -> ViewId {
        let view_id = ViewId(self.id);
        self.id += 1;
        view_id
    }
}

/*
#########################################################
#
# impl LayoutCx
#
#########################################################
*/

impl<'a> LayoutCx<'a> {
    pub fn new(
        cx: &'a mut Context,
        rules: LayoutRules,
        bound: Rect,
    ) -> Self {
        Self {
            cx,
            bound,
            rules,
        }
    }

    pub fn set_node(&mut self, rect: Rect) {
        let id = self.get_id().copied().unwrap();

        if let Some(r) = self.cx.layout_nodes.get_mut(id.0 as usize) {
            *r = rect;
        } else {
            self.cx.layout_nodes.push(rect);
        }
    }

    pub fn get_layout_node(&self) -> Option<&Rect> {
        self.get_id()
            .map(|id| &self.cx.layout_nodes[id.0 as usize])
    }

    pub fn get_available_space(&self) -> Size {
        self.bound.size()
    }
}

/*
#########################################################
#
# impl CursorCx
#
#########################################################
*/

impl<'a> CursorCx<'a> {
    pub fn new(cx: &'a mut Context) -> Self {
        Self {
            cx,
        }
    }

    pub fn get_layout_node(&self) -> Option<&Rect> {
        self.get_id()
            .map(|id| &self.cx.layout_nodes[id.0 as usize])
    }

    pub fn hover_pos(&self) -> &Vec2f {
        &self.cx.cursor.hover.pos
    }

    pub fn set_callback(&mut self, callback: Option<*const dyn Fn()>) {
        self.cx.cursor.captured_callback = callback;
    }

    pub fn set_id(&mut self) {
        self.cx.cursor.hover.curr = self.get_id().copied();
    }
}

/*
#########################################################
#
# impl ViewPath & ViewId
#
#########################################################
*/

impl ViewPath {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
}

impl From<&[u32]> for ViewId {
    fn from(array: &[u32]) -> Self {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        Hash::hash_slice(array, &mut hasher);
        Self(hasher.finish())
    }
}
