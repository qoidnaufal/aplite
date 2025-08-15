use std::sync::OnceLock;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::Vec2f;
use aplite_storage::{IndexMap, entity, Entity};

use crate::view::{Render, View};
use crate::widget::{CALLBACKS, Widget, WidgetEvent, WidgetId};
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::LayoutCx;

pub(crate) static DIRTY: OnceLock<Signal<bool>> = OnceLock::new();

entity! { pub ViewId }

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Event {
    Layout,
    Callback(WidgetId),
    Render,
}

// FIXME: use this as the main building block to build the widget
pub struct Context {
    view_storage: IndexMap<ViewId, View>,
    cursor: Cursor,
    current: Option<ViewId>,
    pending_event: Vec<Event>,
    dirty: Signal<bool>,
}

impl Default for Context {
    fn default() -> Self {
        let dirty = Signal::new(false);
        DIRTY.set(dirty).expect("Should only be initialized once");
        Self {
            view_storage: IndexMap::with_capacity(1024),
            cursor: Cursor::default(),
            current: None,
            pending_event: Vec::with_capacity(16),
            dirty,
        }
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

    pub(crate) fn dirty(&self) -> Signal<bool> {
        self.dirty
    }

    pub(crate) fn toggle_dirty(&self) {
        self.dirty.set(true);
    }

    pub(crate) fn process_pending_update(&mut self) {
        self.pending_event
            .drain(..)
            .for_each(|event| {
                match event {
                    Event::Layout => {},
                    Event::Callback(widget_id) => {
                        CALLBACKS.with(|cb| {
                            if let Some(callbacks) = cb.borrow_mut().get_mut(&widget_id)
                                && let MouseButton::Left = self.cursor.state.button
                                && let Some(callback) = callbacks.get_mut(WidgetEvent::LeftClick)
                            {
                                callback();
                            }
                        })
                    },
                    Event::Render => {},
                }
            });
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

        self.handle_drag();

        self.current = prev;
    }

    fn detect_hover(&mut self) {
        let current = self.current.unwrap();
        let view = self.get_view_ref(&current).unwrap();
        let hovered = view.mouse_hover(&self.cursor);

        self.cursor.hover.curr = hovered;
    }

    pub(crate) fn handle_click(
        &mut self,
        id: &ViewId,
        action: impl Into<MouseAction>,
        button: impl Into<MouseButton>
    ) {
        match self.cursor.process_click_event(action.into(), button.into()) {
            EmittedClickEvent::Captured(widget_id) => {
                let view = self.view_storage.get(id).unwrap();
                let widget = view.find(&widget_id).unwrap();
                let node = widget.node();
                let pos = node.borrow().rect.vec2f();

                self.cursor.click.offset = self.cursor.click.pos - pos;
            },
            EmittedClickEvent::TriggerCallback(widget_id) => {
                self.pending_event.push(Event::Callback(widget_id));
            },
            _ => {}
        }
    }

    pub(crate) fn handle_drag(&mut self) {
        if let Some(view_id) = self.current
            && let Some(captured) = self.cursor.click.captured
        {
            let dragable = self.view_storage
                .get(&view_id)
                .unwrap()
                .find(&captured)
                .unwrap()
                .node()
                .borrow()
                .dragable;

            if self.cursor.is_dragging() && dragable {
                self.cursor.is_dragging = true;
                let pos = self.cursor.hover.pos - self.cursor.click.offset;

                let current = self.view_storage
                    .get_mut(&self.current.unwrap())
                    .unwrap()
                    .find_mut(&captured)
                    .unwrap();

                let node = current.node();
                let mut state = node.borrow_mut();
                state.rect.set_pos(pos.into());

                drop(state);

                if let Some(children) = current.children_ref() {
                    let mut cx = LayoutCx::new(&current);
                    children.iter()
                        .for_each(|child| child.calculate_layout(&mut cx));
                }

                self.toggle_dirty();
            }
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
