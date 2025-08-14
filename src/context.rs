use std::sync::OnceLock;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::Vec2f;
use aplite_storage::IndexMap;

use crate::view::{Render, ViewId, View};
use crate::widget::{CALLBACKS, Widget, WidgetEvent, WidgetId};

pub(crate) mod cursor;
pub mod layout;

use cursor::{Cursor, MouseAction, MouseButton};
use layout::LayoutCx;

pub(crate) static DIRTY: OnceLock<Signal<bool>> = OnceLock::new();

// FIXME: use this as the main building block to build the widget
pub struct Context {
    view_storage: IndexMap<ViewId, View>,
    cursor: Cursor,
    current: Option<ViewId>,
    pending_event: Vec<WidgetId>,
    dirty: Signal<bool>,
}

impl Default for Context {
    fn default() -> Self {
        let dirty = Signal::new(false);
        DIRTY.set(dirty).expect("Should only be initialized once");
        Self {
            view_storage: IndexMap::with_capacity(1024),
            cursor: Cursor::new(),
            current: None,
            pending_event: Vec::with_capacity(16),
            dirty,
        }
    }
}

impl Context {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

// ########################################################
// #                                                      #
// #                        Data                          #
// #                                                      #
// ########################################################

impl Context {
    pub(crate) fn insert_view(&mut self, view: View) -> ViewId {
        self.view_storage.insert(view)
    }

    pub(crate) fn get_view_ref(&self, id: &ViewId) -> Option<&View> {
        self.view_storage.get(id)
    }

    // pub(crate) fn get_view_mut(&mut self, id: &ViewId) -> Option<&mut View> {
    //     self.view_storage.get_mut(id)
    // }

    pub(crate) fn dirty(&self) -> Signal<bool> {
        self.dirty
    }

    pub(crate) fn toggle_dirty(&self) {
        self.dirty.set(true);
    }

    // pub(crate) fn toggle_clean(&self) {
    //     VIEW_STORAGE.with(|s| s.dirty.set(None))
    // }
}

// #########################################################
// #                                                       #
// #                        Layout                         #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn layout(&self, id: &ViewId) {
        // calculate_size_recursive(id);
        if let Some(view) = self.get_view_ref(id) {
            view.calculate_size(None);
            let mut cx = LayoutCx::new(view);
            view.calculate_layout(&mut cx);
        }
        self.toggle_dirty();
    }
}

// #########################################################
// #                                                       #
// #                     Cursor Event                      #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn handle_mouse_move(&mut self, id: &ViewId, pos: impl Into<Vec2f>) {
        if self.get_view_ref(id).is_none() { return }

        let prev = self.current.replace(*id);
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover();
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_hover();

        self.current = prev;
    }

    fn detect_hover(&mut self) {
        if !self.cursor.is_clicking() && self.current.is_some() {
            let current = self.current.unwrap();
            let view = self.get_view_ref(&current).unwrap();
            let hovered = view.mouse_hover(&self.cursor);

            match hovered {
                Some(id) => self.cursor.hover.prev = self.cursor.hover.curr.replace(id),
                None => self.cursor.hover.prev = self.cursor.hover.curr.take(),
            }
        }
    }

    pub(crate) fn handle_hover(&mut self) {
        if !self.cursor.is_idling() && self.current.is_some() {
            if let Some(hover_id) = self.cursor.hover.curr {
                let dragable = self.view_storage
                    .get(&self.current.unwrap())
                    .unwrap()
                    .find(&hover_id)
                    .unwrap()
                    .node()
                    .borrow()
                    .dragable;

                if self.cursor.is_dragging(&hover_id) && dragable {
                    self.cursor.is_dragging = true;
                    self.handle_drag(&hover_id);
                }
            }
        }
    }

    fn handle_drag(&mut self, hover_id: &WidgetId) {
        let pos = self.cursor.hover.pos - self.cursor.click.offset;

        let current = self.view_storage
            .get_mut(&self.current.unwrap())
            .unwrap()
            .find_mut(hover_id)
            .unwrap();

        let node = current.node();
        let mut state = node.borrow_mut();
        state.rect.set_pos(pos.into());

        drop(state);

        if let Some(children) = current.children_ref() {
            let mut cx = LayoutCx::new(&current);
            children.iter()
                .for_each(|child| child.layout(&mut cx));
        }

        self.toggle_dirty();
    }

    pub(crate) fn handle_click(&mut self, id: &ViewId, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        let action: MouseAction = action.into();
        let button: MouseButton = button.into();

        self.cursor.set_click_state(action, button);

        let view = self.view_storage.get(id).unwrap();

        if let Some(hover_id) = self.cursor.hover.curr.as_ref()
            && action == MouseAction::Pressed
            && button == MouseButton::Left
        {
            let widget = view.find(hover_id).unwrap();
            let node = widget.node();
            let pos = node.borrow().rect.vec2f();

            self.cursor.click.offset = self.cursor.click.pos - pos;
            node.borrow_mut().event = Some(WidgetEvent::LeftClick);
            self.pending_event.push(*hover_id);
        }

        if self.cursor.state.action == MouseAction::Released {
            self.pending_event
                .drain(..)
                .for_each(|id| {
                    CALLBACKS.with(|cb| {
                        if let Some(callbacks) = cb.borrow_mut().get_mut(&id)
                            && let MouseButton::Left = self.cursor.state.button
                            && let Some(callback) = callbacks.get_mut(WidgetEvent::LeftClick)
                        {
                            callback();
                        }
                    })
                });

            if let Some(hover_id) = self.cursor.hover.curr.as_ref() {
                let widget = view.find(hover_id).unwrap();
                let node = widget.node();
                node.borrow_mut().event = None;
            }

            self.cursor.is_dragging = false;
        }
    }
}

// #########################################################
// #                                                       #
// #                         Render                        #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn render(&self, root_id: ViewId, renderer: &mut Renderer) {
        if let Some(view) = self.view_storage.get(&root_id) {
            view.render(renderer);
        }
    }
}
