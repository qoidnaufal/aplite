pub(crate) mod cursor;
pub mod layout;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::Vec2f;

use crate::view::{VIEW_STORAGE, ViewId};

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
    pub(crate) fn dirty(&self) -> RwSignal<Option<ViewId>> {
        VIEW_STORAGE.with(|s| s.dirty)
    }

    pub(crate) fn toggle_dirty(&self, root_id: Option<ViewId>) {
        VIEW_STORAGE.with(|s| s.dirty.set(root_id))
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
        self.toggle_dirty(Some(*root_id));
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
                        let state = s.get_widget_state(id);
                        state.detect_hover(&self.cursor)
                            .then_some((state.z_index.get_untracked(), *id))
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
                    s.get_widget_state(&hover_id)
                        .dragable
                        .get_untracked()
                });
                if self.cursor.is_dragging(&hover_id) && dragable {
                    self.cursor.is_dragging.set_untracked(true);
                    self.handle_drag(&hover_id);
                }
            }
        }
    }

    fn handle_drag(&mut self, hover_id: &ViewId) {
        let pos = self.cursor.hover.pos - self.cursor.click.offset;
        let state = VIEW_STORAGE.with(|s| s.get_widget_state(hover_id));
        state.rect.update_untracked(|rect| rect.set_pos(pos.into()));
        LayoutContext::new(*hover_id).calculate();
        self.toggle_dirty(state.root_id.get());
    }

    pub(crate) fn handle_click(&mut self, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        self.cursor.set_click_state(action.into(), button.into());
        if let Some(hover_id) = self.cursor.hover.curr.as_ref() {
            VIEW_STORAGE.with(|s| {
                let state = s.get_widget_state(hover_id);
                let rect = state.rect;
                let pos = rect.read_untracked(|rect| rect.pos());
                self.cursor.click.offset = self.cursor.click.pos - pos;
                state.is_clicked.set(true);
            });
        }
        if self.cursor.state.action == MouseAction::Released {
            if let Some(hover_id) = self.cursor.hover.curr.as_ref() {
                VIEW_STORAGE.with(|s| {
                    let state = s.get_widget_state(hover_id);
                    if !self.cursor.is_dragging.get_untracked() {
                        state.trigger_callback.set(true);
                    }
                    state.is_clicked.set(false);
                });
            }

            self.cursor.is_dragging.set_untracked(false);
        }
    }
}

// #########################################################
// #                                                       #
// #                         Render                        #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn prepare_data(&self, root_id: ViewId, renderer: &mut Renderer) {
        VIEW_STORAGE.with(|s| {
            // TODO: let scene = renderer.new_scene();
            let screen = renderer.screen_res();
            let components = s.get_render_components(&root_id, screen);

            components.iter()
                .enumerate()
                .for_each(|(offset, (elem, mat, img))| {
                    let info = img.as_ref().and_then(|image_fn| renderer.render_image(image_fn));
                    if let Some(id) = info {
                        elem.borrow_mut().set_atlas_id(id);
                    };
                    renderer.submit_data(*elem.borrow(), *mat, offset as _);
                });
        });
        // TODO: renderer.encode_scene(scene);
    }
}
