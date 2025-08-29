use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::Vec2f;

use crate::view::{View, Layout};
use crate::widget::{CALLBACKS, Widget, WidgetEvent};
use crate::state::WidgetId;
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::LayoutCx;
use crate::state::NODE_STORAGE;

// entity! { pub ViewId }

pub struct Context {
    pub(crate) view: View,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pending_update: Vec<WidgetId>,
}

// ########################################################
// #                                                      #
// #                        Data                          #
// #                                                      #
// ########################################################

impl Context {
    pub(crate) fn new(view: View) -> Self {
        Self {
            view,
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            pending_update: Vec::new(),
        }
    }

    pub(crate) fn toggle_dirty(&self) {
        self.dirty.set(true);
    }

    pub(crate) fn process_pending_update(&mut self) {
        NODE_STORAGE.with_borrow(|s| {
            s.iter()
                .filter_map(|(id, state)| {
                    let id = state.borrow().flag.needs_layout().then_some(id);
                    state.borrow_mut().flag.set_needs_layout(false);
                    id
                })
                .for_each(|id| self.pending_update.push(id));
        });

        if !self.pending_update.is_empty() {
            self.pending_update
                .drain(..)
                .for_each(|id| {
                    if let Some(widget) = self.view.find_parent(&id)
                        && let Some(children) = widget.children_ref()
                    {
                        widget.calculate_size(None);
                        let mut cx = LayoutCx::new(widget.as_ref());
                        children.iter()
                            .for_each(|child| child.calculate_layout(&mut cx));
                    }
                });
            self.toggle_dirty();
        }
    }

// #########################################################
// #                                                       #
// #                     Cursor Event                      #
// #                                                       #
// #########################################################

    pub(crate) fn handle_mouse_move(&mut self, pos: impl Into<Vec2f>) {
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover();
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_drag();
    }

    fn detect_hover(&mut self) {
        let hovered = self.view.detect_hover(&self.cursor).map(Widget::id);

        self.cursor.hover.curr = hovered;
    }

    pub(crate) fn handle_drag(&mut self) {
        if let Some(captured) = self.cursor.click.captured {
            NODE_STORAGE.with_borrow(|s| {
                let node = s.get(&captured).unwrap();
                let dragable = node.borrow().flag.is_dragable();

                if self.cursor.is_dragging() && dragable {
                    self.cursor.is_dragging = true;
                    let pos = self.cursor.hover.pos - self.cursor.click.offset;

                    let mut state = node.borrow_mut();
                    state.rect.set_pos(pos);
                    state.flag.set_dirty(true);

                    drop(state);

                    let widget = self.view.find_visible(&captured).unwrap();
                    if let Some(children) = widget.children_ref() {
                        let mut cx = LayoutCx::new(widget);
                        children.iter()
                            .for_each(|child| child.calculate_layout(&mut cx));
                    }

                    self.toggle_dirty();
                }
            });
        }
    }

    pub(crate) fn handle_click(
        &mut self,
        action: impl Into<MouseAction>,
        button: impl Into<MouseButton>
    ) {
        match self.cursor.process_click_event(action.into(), button.into()) {
            EmittedClickEvent::Captured(captured) => {
                NODE_STORAGE.with_borrow(|s| {
                    let node = s.get(&captured).unwrap();
                    let pos = node.borrow().rect.vec2f();

                    self.cursor.click.offset = self.cursor.click.pos - pos;
                })
            },
            EmittedClickEvent::TriggerCallback(id) => {
                CALLBACKS.with_borrow_mut(|cb| {
                    if let Some(callbacks) = cb.get_mut(&id)
                        && let MouseButton::Left = self.cursor.state.button
                        && let Some(callback) = callbacks.get_mut(WidgetEvent::LeftClick)
                    {
                        callback();
                        self.toggle_dirty();
                    }
                });
            },
            _ => {}
        }
    }


// #########################################################
// #                                                       #
// #                         Render                        #
// #                                                       #
// #########################################################

    pub(crate) fn render(&self, renderer: &mut Renderer) {
        let mut scene = renderer.scene();
        self.view.widget.draw(&mut scene);
    }
}
