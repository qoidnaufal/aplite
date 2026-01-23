use std::ptr::NonNull;
use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHasher};
use aplite_renderer::Renderer;
use aplite_types::{Rect, Size, Vec2f};

use crate::layout::{AlignH, AlignV, Axis, LayoutRules, Padding, Spacing};
use crate::cursor::{Cursor, EmittedClickEvent, MouseAction, MouseButton};
use crate::widget::{Renderable, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(pub(crate) u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewPath(pub(crate) Vec<u32>);

pub struct BuildCx<'a> {
    view_path: &'a mut ViewPath,
    view_ids: &'a mut FxHashMap<PathId, ViewId>,
    elements: &'a mut Vec<Box<dyn Renderable>>,
    next_id: u64,
}

pub struct LayoutCx<'a> {
    view_path: &'a mut ViewPath,
    view_ids: &'a mut FxHashMap<PathId, ViewId>,
    elements: &'a mut Vec<Box<dyn Renderable>>,
    layout_nodes: &'a mut Vec<Rect>,
    pub(crate) bound: Rect,
    pub(crate) rules: LayoutRules,
}

pub struct CursorCx<'a> {
    view_path: &'a mut ViewPath,
    view_ids: &'a mut FxHashMap<PathId, ViewId>,
    elements: &'a mut Vec<Box<dyn Renderable>>,
    cursor: &'a mut Cursor,
    layout_nodes: &'a mut Vec<Rect>,
}

pub struct Context {
    pub(crate) elements: Vec<Box<dyn Renderable>>,
    pub(crate) layout_nodes: Vec<Rect>,
    view_ids: FxHashMap<PathId, ViewId>,
    pub(crate) view_path: ViewPath,
    pub(crate) cursor: Cursor,
    pub(crate) window_rect: Rect,
    pub(crate) redraw_phase: bool,
}

impl Context {
    pub(crate) fn new(size: Size) -> Self {
        Self {
            elements: Vec::new(),
            layout_nodes: Vec::new(),
            view_ids: FxHashMap::default(),
            view_path: ViewPath::new(),
            cursor: Cursor::default(),
            window_rect: Rect::from_size(size),
            redraw_phase: false,
        }
    }

    pub fn build<T: Widget>(&mut self, view: &T) -> bool {
        if self.redraw_phase {
            self.redraw_phase = false;
            return false;
        }

        let mut cx = BuildCx::new(self);
        let dirty = cx.with_id(0, |cx| view.build(cx));

        let count = cx.next_id as usize;
        let len = self.elements.len();
        self.elements.truncate(count);

        let dirty = dirty || count != len;
        self.redraw_phase = dirty;
        dirty
    }

    pub fn rebuild<T: Widget>(&mut self, view: &T) -> bool {
        if self.build(view) {
            self.layout(view);
            return true;
        }

        false
    }
 
    pub fn layout<T: Widget>(&mut self, view: &T) {
        let rules = LayoutRules {
            padding: Padding::default(),
            axis: Axis::Vertical,
            align_h: AlignH::Left,
            align_v: AlignV::Top,
            spacing: Spacing(0),
        };

        let mut cx = LayoutCx::new(self, rules, self.window_rect);
        cx.with_id(0, |cx| view.layout(cx));

        let len = self.elements.len();
        self.layout_nodes.truncate(len);
    }

    pub(crate) fn handle_mouse_move<T: Widget>(&mut self, pos: impl Into<Vec2f>, view: &T) {
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        let mut cx = CursorCx::new(self);
        cx.with_id(0, |cx| view.detect_hover(cx));
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_drag();
    }

    pub(crate) fn handle_drag(&mut self) {
        if self.cursor.is_dragging() {
            if let Some(captured) = self.cursor.captured.id {
                if !self.cursor.hover.curr.is_some_and(|id| id == captured) {
                    self.cursor.captured.callback = None;
                }

                // let pos = self.cursor.hover.pos - self.cursor.click.offset;
                // let node = &mut self.layout_nodes[captured.0 as usize];
                // node.set_pos(pos);
                // self.toggle_dirty();
            }
        }
    }

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
                    let cb = &*callback.as_ptr();
                    cb()
                }
            },
            _ => {}
        }
    }

    pub(crate) fn render(&self, renderer: &mut Renderer) {
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
                self.view_path.0.pop().unwrap_or_default()
            }

            pub(crate) fn push(&mut self, path_id: u32) {
                self.view_path.0.push(path_id);
            }

            pub fn with_id<R: 'static>(&mut self, id_path: u32, f: impl FnOnce(&mut Self) -> R) -> R {
                self.push(id_path);
                let res = f(self);
                self.pop();
                res
            }

            pub fn get_id(&self) -> Option<&ViewId> {
                let path_id = self.view_path.get_path_id();
                self.view_ids.get(&path_id)
            }

            pub fn get_parent_id(&self) -> Option<&ViewId> {
                self.view_path.get_parent_path_id()
                    .and_then(|id| self.view_ids.get(&id))
            }

            pub fn get_z_index(&self) -> u32 {
                self.view_path.0.len() as u32
            }

            pub fn get_element<S: Renderable + 'static>(&self) -> Option<&S> {
                self.get_id()
                    .and_then(|id| unsafe {
                        let ptr = self.elements[id.0 as usize].as_ref() as *const dyn Renderable;
                        if (&*ptr).type_id() == std::any::TypeId::of::<S>() {
                            Some(&*ptr.cast::<S>())
                        } else {
                            None
                        }
                    })
            }

            pub fn get_element_mut<S: Renderable + 'static>(&mut self) -> Option<&mut S> {
                self.get_id()
                    .copied()
                    .and_then(|id| unsafe {
                        let ptr = self.elements[id.0 as usize].as_mut() as *mut dyn Renderable;
                        if (&*ptr).type_id() == std::any::TypeId::of::<S>() {
                            Some(&mut *ptr.cast::<S>())
                        } else {
                            None
                        }
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
            view_path: &mut cx.view_path,
            view_ids: &mut cx.view_ids,
            elements: &mut cx.elements,
            next_id: 0,
        }
    }

    #[must_use]
    pub fn register_element<R: Renderable + 'static>(&mut self, element: R) -> bool {
        let id = self.get_or_create_id();

        if let Some(exist) = self.elements.get_mut(id.0 as usize) {
            if exist.equal(&element) {
                return false;
            }
            *exist = Box::new(element);
        } else {
            self.elements.push(Box::new(element));
        }

        true
    }

    fn get_or_create_id(&mut self) -> ViewId {
        let path_id = self.view_path.get_path_id();

        let view_id = if let Some(view_id) = self.view_ids.get(&path_id) {
            *view_id
        } else {
            let view_id = ViewId(self.next_id);
            self.view_ids.insert(path_id, view_id);
            view_id
        };

        self.next_id += 1;
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
    pub fn new(cx: &'a mut Context, rules: LayoutRules, bound: Rect) -> Self {
        Self {
            view_path: &mut cx.view_path,
            view_ids: &mut cx.view_ids,
            elements: &mut cx.elements,
            layout_nodes: &mut cx.layout_nodes,
            rules,
            bound,
        }
    }

    pub fn derive<'b: 'a>(
        prev: &'b mut LayoutCx<'_>,
        rules: LayoutRules,
        bound: Rect
    ) -> Self {
        Self {
            view_path: prev.view_path,
            view_ids: prev.view_ids,
            elements: prev.elements,
            layout_nodes: prev.layout_nodes,
            bound,
            rules,
        }
    }

    pub fn set_node(&mut self, rect: Rect) {
        let id = self.get_id().copied().unwrap();

        if let Some(r) = self.layout_nodes.get_mut(id.0 as usize) {
            *r = rect;
        } else {
            self.layout_nodes.push(rect);
        }
    }

    pub fn get_layout_node(&self) -> Option<&Rect> {
        self.get_id()
            .map(|id| &self.layout_nodes[id.0 as usize])
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
            view_path: &mut cx.view_path,
            view_ids: &mut cx.view_ids,
            elements: &mut cx.elements,
            layout_nodes: &mut cx.layout_nodes,
            cursor: &mut cx.cursor,
        }
    }

    pub fn get_layout_node(&self) -> Option<&Rect> {
        self.get_id()
            .map(|id| &self.layout_nodes[id.0 as usize])
    }

    pub fn hover_pos(&self) -> &Vec2f {
        &self.cursor.hover.pos
    }

    pub fn set_callback_on_click<F>(&mut self, callback: F)
    where
        F: FnOnce() -> NonNull<dyn Fn()>
    {
        if self.cursor.is_left_clicking() {
            self.cursor.captured.callback = Some(callback());
        }
    }

    pub fn set_id(&mut self) {
        self.cursor.hover.curr = self.get_id().copied();
    }

    pub fn is_clicking(&self) -> bool {
        self.cursor.is_left_clicking()
    }
}

/*
#########################################################
#
# impl ViewPath & ViewId
#
#########################################################
*/

#[derive(Debug, PartialEq, Eq)]
struct PathId(u64);

impl Hash for PathId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl ViewPath {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    fn get_path_id(&self) -> PathId {
        let mut hasher = FxHasher::default();
        Hash::hash_slice(&self.0, &mut hasher);
        PathId(hasher.finish())
    }

    fn get_parent_path_id(&self) -> Option<PathId> {
        if self.0.len() <= 1 { return None; }
        let mut hasher = FxHasher::default();
        Hash::hash_slice(&self.0[..self.0.len() - 1], &mut hasher);
        Some(PathId(hasher.finish()))
    }
}
