use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::Vec2f;
use aplite_storage::{IndexMap, entity, Entity};

use crate::view::{View, Layout};
use crate::widget::{CALLBACKS, Widget, WidgetEvent};
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::LayoutCx;
use crate::state::{WidgetId, NODE_STORAGE};

entity! { pub ViewId }

pub struct Context {
    view_storage: IndexMap<ViewId, View>,
    cursor: Cursor,
    dirty: Signal<bool>,
    needs_layout: Vec<WidgetId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            view_storage: IndexMap::with_capacity(1024),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            needs_layout: Vec::new(),
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

    pub(crate) fn process_pending_update(&mut self, view_id: &ViewId) {
        if let Some(view) = self.view_storage.get(view_id) {
            NODE_STORAGE.with_borrow(|s| {
                s.iter()
                    .filter_map(|(id, state)| {
                        let id = state.borrow().flag.needs_layout().then_some(id);
                        state.borrow_mut().flag.set_needs_layout(false);
                        id
                    })
                    .for_each(|id| self.needs_layout.push(id));
            });

            if !self.needs_layout.is_empty() {
                self.needs_layout
                    .drain(..)
                    .for_each(|id| {
                        if let Some(widget) = view.find_parent(&id)
                            && let Some(children) = widget.children_ref()
                        {
                            widget.calculate_size(None);
                            let mut cx = LayoutCx::new(&widget);
                            children.iter()
                                .for_each(|child| child.calculate_layout(&mut cx));
                        }
                    });
                self.toggle_dirty();
            }
        }
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
        let hovered = view.detect_hover(&self.cursor);

        self.cursor.hover.curr = hovered;
    }

    pub(crate) fn handle_drag(&mut self) {
        if let Some(captured) = self.cursor.click.captured {
            let node = captured.node_ref().upgrade();
            let dragable = node.borrow().flag.is_dragable();

            if self.cursor.is_dragging() && dragable {
                self.cursor.is_dragging = true;
                let pos = self.cursor.hover.pos - self.cursor.click.offset;

                let mut state = node.borrow_mut();
                state.rect.set_pos(pos);
                state.flag.set_dirty(true);

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
                CALLBACKS.with_borrow_mut(|cb| {
                    if let Some(callbacks) = cb.get_mut(&widget.id())
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
