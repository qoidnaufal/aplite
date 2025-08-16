use std::sync::OnceLock;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::{Vec2f, Size, Rect};
use aplite_storage::{IndexMap, entity, Entity};

use crate::view::{Render, View, Layout};
use crate::widget::{CALLBACKS, Widget, WidgetEvent, WindowWidget};
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::LayoutCx;

pub(crate) static PENDING_EVENT: OnceLock<SignalWrite<Vec<Event>>> = OnceLock::new();

entity! { pub ViewId }

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Event {
    Layout,
    Callback(*const dyn Widget),
    Paint,
    Render,
}

// FIXME: use this as the main building block to build the widget
pub struct Context {
    view_storage: IndexMap<ViewId, View>,
    cursor: Cursor,
    current: Option<ViewId>,
    pending_event: Signal<Vec<Event>>,
    dirty: Signal<bool>,
}

impl Default for Context {
    fn default() -> Self {
        let pending_event = Signal::new(Vec::with_capacity(16));
        PENDING_EVENT
            .set(pending_event.write_only())
            .expect("Should only be initialized once");

        Self {
            view_storage: IndexMap::with_capacity(1024),
            cursor: Cursor::default(),
            current: None,
            pending_event,
            dirty: Signal::new(false),
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

    pub(crate) fn process_pending_update(&mut self, view_id: ViewId, size: Size) {
        if self.pending_event.with_untracked(|vec| vec.is_empty()) { return }

        let prev = self.current.replace(view_id);

        self.pending_event
            .update_untracked(|vec| {
                vec
                    .drain(..)
                    .for_each(|event| {
                        match event {
                            Event::Callback(widget) => {
                                // let widget = unsafe { widget.as_ref().unwrap() };
                                CALLBACKS.with(|cb| {
                                    if let Some(callbacks) = cb.borrow_mut().get_mut(&widget.id())
                                        && let MouseButton::Left = self.cursor.state.button
                                        && let Some(callback) = callbacks.get_mut(WidgetEvent::LeftClick)
                                    {
                                        callback();
                                    }
                                })
                            },
                            Event::Layout => {
                                if let Some(current) = self.current
                                    && let Some(view) = self.view_storage.get(&current) {
                                        // FIXME: improve dynamic layouting
                                        view.calculate_size(None);
                                        let window_widget = WindowWidget::new(Rect::from_size(size));
                                        let mut cx = LayoutCx::new(&window_widget);
                                        view.calculate_layout(&mut cx);
                                    }
                            },
                            _ => {},
                        }

                        self.toggle_dirty();
                    });
            });

        self.current = prev;
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

    pub(crate) fn handle_drag(&mut self) {
        if let Some(captured) = self.cursor.click.captured {
            let node = captured.node();
            let dragable = node.borrow().dragable;

            if self.cursor.is_dragging() && dragable {
                self.cursor.is_dragging = true;
                let pos = self.cursor.hover.pos - self.cursor.click.offset;

                let mut state = node.borrow_mut();
                state.rect.set_pos(pos.into());

                drop(state);

                if let Some(children) = captured.children_ref() {
                    let mut cx = LayoutCx::new(&captured);
                    children.iter()
                        .for_each(|child| child.calculate_layout(&mut cx));
                }

                self.toggle_dirty();
            }
        }
    }

    pub(crate) fn handle_click(
        &mut self,
        action: impl Into<MouseAction>,
        button: impl Into<MouseButton>
    ) {
        match self.cursor.process_click_event(action.into(), button.into()) {
            EmittedClickEvent::Captured(widget) => {
                let widget = unsafe { widget.as_ref().unwrap() };
                let node = widget.node();
                let pos = node.borrow().rect.vec2f();

                self.cursor.click.offset = self.cursor.click.pos - pos;
            },
            EmittedClickEvent::TriggerCallback(widget) => {
                self.pending_event.update(|vec| vec.push(Event::Callback(widget)));
            },
            _ => {}
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
