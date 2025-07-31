pub(crate) mod cursor;
pub mod layout;

use aplite_reactive::*;
use aplite_renderer::Scene;
use aplite_types::Vec2f;

use crate::view::{VIEW_STORAGE, ViewId};
use crate::widget::WidgetEvent;

use cursor::{Cursor, MouseAction, MouseButton};
use layout::{LayoutContext, calculate_size_recursive};

// FIXME: use this as the main building block to build the widget
pub struct Context {
    cursor: Cursor,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            cursor: Cursor::new(),
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
                s.hoverable
                    .borrow()
                    .iter()
                    .filter_map(|id| {
                        let tree = s.tree.borrow();
                        let state = tree.get(id).unwrap();
                        state.detect_hover(&self.cursor)
                            .then_some((state.z_index, *id))
                    }).max()
            });

            match hovered {
                Some((z_index, id)) => {
                    self.cursor.hover.prev = self.cursor.hover.curr.replace(id);
                    self.cursor.hover.z_index = z_index;
                },
                None => {
                    self.cursor.hover.prev = self.cursor.hover.curr.take();
                    self.cursor.hover.z_index = 0;
                },
            }
        }
    }

    pub(crate) fn handle_hover(&mut self) {
        if !self.cursor.is_idling() {
            if let Some(hover_id) = self.cursor.hover.curr {
                let dragable = VIEW_STORAGE.with(|s| {
                    s.tree
                        .borrow()
                        .get(&hover_id)
                        .unwrap()
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
            let mut tree = s.tree.borrow_mut();
            let state = tree.get_mut(hover_id).unwrap();
            state.rect.set_pos(pos.into());
            LayoutContext::new(*hover_id).calculate();
            Self::toggle_dirty();
        });
    }

    pub(crate) fn handle_click(&mut self, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        self.cursor.set_click_state(action.into(), button.into());
        if let Some(hover_id) = self.cursor.hover.curr.as_ref() {
            VIEW_STORAGE.with(|s| {
                let mut tree = s.tree.borrow_mut();
                let state = tree.get_mut(hover_id).unwrap();
                let pos = state.rect.pos();
                self.cursor.click.offset = self.cursor.click.pos - pos;
                state.event = Some(WidgetEvent::LeftClick);
            });
        }
        if self.cursor.state.action == MouseAction::Released {
            if let Some(hover_id) = self.cursor.hover.curr.as_ref() {
                VIEW_STORAGE.with(|s| {
                    let mut tree = s.tree.borrow_mut();
                    let state = tree.get_mut(hover_id).unwrap();
                    if !self.cursor.is_dragging {
                        // state.trigger_callback = true;
                    }
                    state.event = None;
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
    pub(crate) fn prepare_data(&self, root_id: ViewId, mut scene: Scene<'_>) {
        VIEW_STORAGE.with(|s| {
            s.get_all_members_of(&root_id)
                .iter()
                .for_each(|view_id| {
                    let size = scene.size();
                    let tree = s.tree.borrow();
                    let state = tree.get(view_id).unwrap();

                    let background = state.background.as_paint_ref();
                    let border = state.border_color.as_paint_ref();
                    let shape = state.shape;
                    let transform = state.get_transform(size);
                    let border_width = if state.border_width == 0.0 {
                        5.0 / size.width
                    } else {
                        state.border_width / size.width
                    };

                    scene.draw(transform, background, border, border_width, shape);
                })
        });
    }
}
