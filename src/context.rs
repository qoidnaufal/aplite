use std::sync::OnceLock;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::{Vec2f, Size, Rect};
use aplite_storage::{IndexMap, entity, Entity};

use crate::view::{View, Layout};
use crate::widget::{CALLBACKS, Widget, WidgetEvent, WindowWidget};
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::LayoutCx;

pub(crate) static PENDING_EVENT: OnceLock<Signal<Vec<Event>>> = OnceLock::new();

entity! { pub ViewId }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Event {
    Layout,
    Callback(*const dyn Widget),
    Paint,
}

// FIXME: use this as the main building block to build the widget
pub struct Context {
    view_storage: IndexMap<ViewId, View>,
    cursor: Cursor,
    dirty: Signal<bool>,
}

impl Default for Context {
    fn default() -> Self {
        PENDING_EVENT
            .set(Signal::new(Vec::with_capacity(16)))
            .expect("Should only be initialized once");

        Self {
            view_storage: IndexMap::with_capacity(1024),
            cursor: Cursor::default(),
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
        let pending_event = *PENDING_EVENT.get().expect("Pending event should have been initialized");
        if pending_event.with_untracked(|vec| vec.is_empty()) { return }

        pending_event.update_untracked(|vec| {
            vec.drain(..).for_each(|event| {
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
                        if let Some(view) = self.view_storage.get(&view_id) {
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

        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover(id);
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_drag();
    }

    fn detect_hover(&mut self, id: &ViewId) {
        let view = self.get_view_ref(id).unwrap();
        let hovered = view.mouse_hover(&self.cursor);

        self.cursor.hover.curr = hovered;
    }

    pub(crate) fn handle_drag(&mut self) {
        if let Some(captured) = self.cursor.click.captured {
            let node = captured.node_ref().upgrade();
            let dragable = node.borrow().is_dragable();

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
                let node = widget.node_ref().upgrade();
                let pos = node.borrow().rect.vec2f();

                self.cursor.click.offset = self.cursor.click.pos - pos;
            },
            EmittedClickEvent::TriggerCallback(widget) => {
                let pending_event = *PENDING_EVENT.get()
                    .expect("Pending event should have been initialized");
                pending_event.update(|vec| vec.push(Event::Callback(widget)));
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
    pub(crate) fn render(&self, root_id: &ViewId, renderer: &mut Renderer) {
        if let Some(view) = self.view_storage.get(root_id) {
            let mut scene = renderer.scene();
            view.draw(&mut scene);
        }
    }
}
