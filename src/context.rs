use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::Vec2f;

use crate::view::{VIEW_STORAGE, ViewId};
use crate::widget::{CALLBACKS, WidgetEvent, Widget};

pub(crate) mod cursor;
pub mod layout;

use cursor::{Cursor, MouseAction, MouseButton};
use layout::{LayoutContext, calculate_size_recursive};

// FIXME: use this as the main building block to build the widget
pub struct Context {
    cursor: Cursor,
    pending_event: Vec<ViewId>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            cursor: Cursor::new(),
            pending_event: Vec::with_capacity(16),
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
    pub(crate) fn dirty() -> Signal<bool> {
        VIEW_STORAGE.with(|s| s.dirty)
    }

    pub(crate) fn toggle_dirty() {
        VIEW_STORAGE.with(|s| s.dirty.set(true))
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
    pub(crate) fn layout_the_whole_window(&self, root_id: &ViewId) {
        calculate_size_recursive(root_id);
        LayoutContext::new(*root_id).calculate();
        Self::toggle_dirty();
    }
}

// #########################################################
// #                                                       #
// #                     Cursor Event                      #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn handle_mouse_move(&mut self, root_id: &ViewId, pos: impl Into<Vec2f>) {
        if VIEW_STORAGE.with(|s| s.get_all_members_of(root_id).is_empty()) { return }
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover();
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_hover();
    }

    fn detect_hover(&mut self) {
        if !self.cursor.is_clicking() {
            let hovered = VIEW_STORAGE.with(|s| {
                s.tree
                    .borrow()
                    .iter()
                    .filter_map(|node| {
                        let data = node.data();
                        data.borrow()
                            .hoverable
                            .then_some((node.id(), data.borrow().rect))
                    })
                    .find_map(|(id, rect)| rect
                        .contains(self.cursor.hover.pos)
                        .then_some(id)
                    )
            });

            match hovered {
                Some(id) => {
                    self.cursor.hover.prev = self.cursor.hover.curr.replace(id);
                },
                None => {
                    self.cursor.hover.prev = self.cursor.hover.curr.take();
                },
            }
        }
    }

    pub(crate) fn handle_hover(&mut self) {
        if !self.cursor.is_idling() {
            if let Some(hover_id) = self.cursor.hover.curr {
                let dragable = VIEW_STORAGE.with(|s| {
                    s.get_widget_state(&hover_id)
                        .unwrap()
                        .borrow()
                        .dragable
                });
                if self.cursor.is_dragging(&hover_id) && dragable {
                    self.cursor.is_dragging = true;
                    self.handle_drag(&hover_id);
                }
            }
        }
    }

    fn handle_drag(&mut self, hover_id: &ViewId) {
        let pos = self.cursor.hover.pos - self.cursor.click.offset;
        VIEW_STORAGE.with(|s| {
            let state = s.get_widget_state(hover_id).unwrap();
            state.borrow_mut().rect.set_pos(pos.into());

            drop(state);

            LayoutContext::new(*hover_id).calculate();
            Self::toggle_dirty();
        });
    }

    pub(crate) fn handle_click(&mut self, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        let action: MouseAction = action.into();
        let button: MouseButton = button.into();

        self.cursor.set_click_state(action, button);

        if let Some(hover_id) = self.cursor.hover.curr.as_ref()
            && action == MouseAction::Pressed
            && button == MouseButton::Left
        {
            VIEW_STORAGE.with(|s| {
                let state = s.get_widget_state(hover_id).unwrap();
                let pos = state.borrow().rect.vec2f();
                self.cursor.click.offset = self.cursor.click.pos - pos;
                state.borrow_mut().event = Some(WidgetEvent::LeftClick);
            });
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
                VIEW_STORAGE.with(|s| {
                    let state = s.get_widget_state(hover_id).unwrap();
                    state.borrow_mut().event = None;
                });
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
    pub(crate) fn prepare_data(&self, _root_id: ViewId, renderer: &mut Renderer) {

        VIEW_STORAGE.with(|s| {
            let views = s.views.borrow();
            views.iter().rev().for_each(|view| {
                view.render(renderer);
            });
            // s.get_all_members_of(&root_id)
            //     .iter()
            //     .for_each(|view_id| {
            //         let scene = renderer.scene();

            //         let scene_size = scene.size();
            //         // let state = s.get_widget_state(view_id).unwrap();
            //         let state = crate::state::WidgetState::new();

            //         let background = state.background.as_paint_ref();
            //         let border = state.border_color.as_paint_ref();
            //         let shape = state.shape;
            //         let rotation = state.rotation;
            //         let transform = state.get_transform(scene_size);

            //         let border_width = if state.border_width == 0.0 {
            //             5.0 / scene_size.width
            //         } else {
            //             state.border_width / scene_size.width
            //         };

            //         scene.draw(
            //             transform,
            //             rotation,
            //             background,
            //             border,
            //             border_width,
            //             shape,
            //         );
            //     })
        });
    }
}
