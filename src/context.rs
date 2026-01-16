use std::any::Any;
use std::collections::HashMap;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_storage::SlotId;
use aplite_types::{Rect, Size, Vec2f};

use crate::view::IntoView;
use crate::cursor::{Cursor, MouseAction, MouseButton};
use crate::widget::Widget;
// use crate::callback::CallbackStorage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewId(u64);

pub struct ViewPath(Vec<u32>);

pub struct BuildCx<'a> {
    pub(crate) cx: &'a mut Context,
    pub(crate) id: u64,
    pub(crate) path: ViewPath,
}

pub struct Context {
    pub(crate) states: Vec<Box<dyn Any>>,
    pub(crate) view_ids: HashMap<Box<[u32]>, ViewId>,
    // pub(crate) callbacks: CallbackStorage,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pub(crate) window_rect: Rect,
    pub(crate) pending_update: Vec<SlotId>,
}

impl Context {
    pub(crate) fn new(size: Size) -> Self {
        Self {
            states: Vec::new(),
            view_ids: HashMap::new(),
            // callbacks: CallbackStorage::default(),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            window_rect: Rect::from_size(size),
            pending_update: Vec::new(),
        }
    }

    pub fn build<IV: IntoView>(&mut self, view: &IV) {
        let mut cx = BuildCx::new(self);
        cx.with_id(0, |cx| view.build(cx));

        let len = cx.id as usize;
        self.states.truncate(len);
    }
 
    pub fn layout(&mut self) {}

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

    pub(crate) fn handle_mouse_move(&mut self, pos: impl Into<Vec2f>) {
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover();
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_drag();
    }

    fn detect_hover(&mut self) {
        // let query = self.storage.query::<(&Vec2f, &mut Size)>();
    }

    pub(crate) fn handle_drag(&mut self) {}

    pub(crate) fn handle_click(
        &mut self,
        action: impl Into<MouseAction>,
        button: impl Into<MouseButton>
    ) {
        let _ = action;
        let _ = button;
        // match self.cursor.process_click_event(action.into(), button.into()) {
        //     EmittedClickEvent::Captured(captured) => {
        //         let pos = self.state.common.query::<&Rect>().get(&captured).unwrap().vec2f();
        //         self.cursor.click.offset = self.cursor.click.pos - pos;
        //     },
        //     EmittedClickEvent::TriggerCallback(id) => {
        //         CALLBACKS.with_borrow_mut(|cb| {
        //             if let Some(callbacks) = cb.get_mut(&id)
        //                 && let MouseButton::Left = self.cursor.state.button
        //                 && let Some(callback) = callbacks.get_mut(WidgetEvent::LeftClick)
        //             {
        //                 callback();
        //                 self.toggle_dirty();
        //             }
        //         });
        //     },
        //     _ => {}
        // }
    }

    pub(crate) fn render<W: Widget>(&self, _widget: &W, renderer: &mut Renderer) {
        let _scene = renderer.scene();
    }
}

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
            path: ViewPath::new(),
        }
    }

    pub fn set_state<S: 'static>(&mut self, state: S) {
        let id = self.create_id();
        let path = self.get_path();
        self.cx.view_ids.insert(path, id);

        if let Some(any) = self.cx.states.get_mut(id.0 as usize) {
            *any = Box::new(state);
        } else {
            self.cx.states.push(Box::new(state));
        }
    }

    pub(crate) fn pop(&mut self) -> u32 {
        self.path.0.pop().unwrap_or_default()
    }

    pub(crate) fn push(&mut self, path_id: u32) {
        self.path.0.push(path_id);
    }

    pub fn with_id<R: 'static>(&mut self, id_path: u32, f: impl FnOnce(&mut Self) -> R) -> R {
        self.path.0.push(id_path);
        let res = f(self);
        self.path.0.pop();
        res
    }

    pub fn create_id(&mut self) -> ViewId {
        let view_id = ViewId(self.id);
        self.id += 1;
        view_id
    }

    pub fn get_id(&self) -> Option<&ViewId> {
        let path = self.get_path();
        self.cx.view_ids.get(&path)
    }

    pub fn get_parent_id(&self) -> Option<&ViewId> {
        if self.path.0.is_empty() {
            None
        } else {
            let parent_path = self.path
                .0[..self.path.0.len() - 1]
                .iter()
                .copied()
                .collect::<Box<[_]>>();
            self.cx.view_ids.get(&parent_path)
        }
    }

    pub fn get_path(&self) -> Box<[u32]> {
        self.path.0.clone().into_boxed_slice()
    }

    pub fn get_z_index(&self) -> u32 {
        self.path.0.len() as u32
    }

    pub fn get_state<S: 'static>(&mut self, id: ViewId) -> Option<&mut S> {
        self.cx.states.get_mut(id.0 as usize)
            .and_then(|any| any.downcast_mut())
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
