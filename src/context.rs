use std::collections::HashMap;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_types::{Size, Vector2};
use winit::window::WindowId;

use crate::view::ViewId;
use crate::view::VIEW_STORAGE;

pub mod layout;
pub(crate) mod widget_state;
pub(crate) mod cursor;

use widget_state::AspectRatio;
use cursor::{Cursor, MouseAction, MouseButton};
use layout::{
    LayoutContext,
    Orientation,
};

// i think this one could be a reactive system too
pub struct Context {
    pub(crate) root_window: HashMap<ViewId, WindowId>,
    cursor: Cursor,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            root_window: HashMap::new(),
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
    pub(crate) fn dirty(&self) -> RwSignal<bool> {
        VIEW_STORAGE.with(|s| s.dirty)
    }

    pub(crate) fn toggle_dirty(&self) {
        VIEW_STORAGE.with(|s| s.dirty.set(true))
    }

    pub(crate) fn toggle_clean(&self) {
        VIEW_STORAGE.with(|s| s.dirty.set(false))
    }
}

// #########################################################
// #                                                       #
// #                        Layout                         #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn layout_the_whole_window(&self, root_id: &ViewId) {
        self.calculate_size_recursive(root_id);
        self.recursive_layout_from_id(root_id);
    }

    pub(crate) fn recursive_layout_from_id(&self, id: &ViewId) {
        let children = LayoutContext::new(id).calculate();
        if let Some(children) = children {
            children.iter().for_each(|child| self.recursive_layout_from_id(child));
        }
        self.toggle_dirty();
    }

    fn calculate_size_recursive(&self, id: &ViewId) -> Size<u32> {
        let widget_state = VIEW_STORAGE.with(|s| s.get_widget_state(id));
        let padding = widget_state.padding();
        let mut size = widget_state.rect.read_untracked(|rect| rect.size());

        let mut resized = false;

        let maybe_children = VIEW_STORAGE.with(|s| {
            s.tree.borrow()
                .get_all_children(id)
                .map(|v| v.iter().map(|c| **c).collect::<Vec<_>>())
        });
        if let Some(children) = maybe_children {
            children.iter().for_each(|child_id| {
                let child_size = self.calculate_size_recursive(child_id);
                match widget_state.orientation() {
                    Orientation::Vertical => {
                        size.add_height(child_size.height());
                        size.set_width(size.width().max(child_size.width() + padding.horizontal()));
                    }
                    Orientation::Horizontal => {
                        size.set_height(size.height().max(child_size.height() + padding.vertical()));
                        size.add_width(child_size.width());
                    }
                }
            });
            let child_len = children.len() as u32;
            let stretch = widget_state.spacing() * (child_len - 1);
            match widget_state.orientation() {
                Orientation::Vertical => {
                    size.add_height(padding.vertical() + stretch);
                },
                Orientation::Horizontal => {
                    size.add_width(padding.horizontal() + stretch);
                },
            }
        }

        if let AspectRatio::Defined(tuple) = widget_state.image_aspect_ratio() {
            VIEW_STORAGE.with(|s| {
                if let Some(parent) = s.tree.borrow().get_parent(id) {
                    match s.get_widget_state(parent).orientation() {
                        Orientation::Vertical => size.adjust_height(tuple.into()),
                        Orientation::Horizontal => size.adjust_width(tuple.into()),
                    }
                } else {
                    size.adjust_width(tuple.into());
                }
            });
            widget_state.rect.write_untracked(|rect| rect.set_size(size));
        }

        let final_size = size
            .max(widget_state.min_width(), widget_state.min_height())
            .min(widget_state.max_width(), widget_state.max_height());

        resized |= final_size != widget_state.rect.read_untracked(|rect| rect.size());

        if resized {
            widget_state.rect.write_untracked(|state| state.set_size(final_size));
        }

        final_size
    }
}

// #########################################################
// #                                                       #
// #                     Cursor Event                      #
// #                                                       #
// #########################################################

impl Context {
    pub(crate) fn handle_mouse_move(&mut self, root_id: &ViewId, pos: impl Into<Vector2<f32>>) {
        VIEW_STORAGE.with(|s| {
            if s.get_all_members_of(root_id).is_empty() { return }
        });
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover();
        #[cfg(feature = "cursor_stats")] eprintln!("{:?}", start.elapsed());

        self.handle_hover();
    }

    fn detect_hover(&mut self) {
        if self.cursor.click.obj.is_none() {
            VIEW_STORAGE.with(|s| {
                s.hoverable
                    .borrow()
                    .iter()
                    .for_each(|id| {
                        let state = s.get_widget_state(id);
                        state.detect_hover(&mut self.cursor, id);
                    });
            })
        }
    }

    pub(crate) fn handle_hover(&mut self) {
        if self.cursor.is_idling() || self.cursor.is_unscoped() { return }

        if let Some(hover_id) = self.cursor.hover.curr {
            let dragable = VIEW_STORAGE.with(|s| {
                s.get_widget_state(&hover_id)
                    .dragable
                    .get_untracked()
            });
            if self.cursor.is_dragging(&hover_id) && dragable {
                self.handle_drag(&hover_id);
            }
        }
    }

    fn handle_drag(&mut self, hover_id: &ViewId) {
        let pos = self.cursor.hover.pos - self.cursor.click.offset;
        VIEW_STORAGE.with(|s| s.get_widget_state(hover_id).rect)
            .write_untracked(|rect| rect.set_pos(pos.into()));
        self.recursive_layout_from_id(hover_id);
    }

    pub(crate) fn handle_click(&mut self, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        self.cursor.set_click_state(action.into(), button.into());
        if let Some(click_id) = self.cursor.click.obj.as_ref() {
            VIEW_STORAGE.with(|s| {
                s.invoke_callback(click_id);

                let state = s.get_widget_state(click_id);
                let rect = state.rect;
                let pos = rect.read_untracked(|rect| rect.pos().f32());
                self.cursor.click.offset = self.cursor.click.pos - pos;
                state.is_clicked.write_untracked(|val| *val = true);
            });
        }
        if self.cursor.state.action == MouseAction::Released {
            if let Some(hover_id) = self.cursor.hover.curr.as_ref() {
                VIEW_STORAGE.with(|s| {
                    s.get_widget_state(hover_id)
                        .is_clicked
                        .write_untracked(|val| *val = false);
                })
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
        VIEW_STORAGE.with(|s| {
            renderer.begin();

            let screen = renderer.screen_size();
            let components = s.get_render_components(&root_id, screen);

            let mut elements = Vec::with_capacity(components.len());
            let mut transforms = Vec::with_capacity(components.len());
            let mut mesh = Vec::with_capacity(components.len() * 4);

            components.iter().enumerate().for_each(|(idx, (elem, vert, mat, img))| {
                let info = img.as_ref().map(|image_fn| renderer.push_atlas(image_fn)).flatten();
                if let Some(info) = info {
                    if let Some(uv) = info.get_uv() {
                        vert.write_untracked(|vertices| vertices.set_uv(uv));
                    }
                    let atlas_id = info.get_atlas_id().unwrap_or(-1);
                    let image_id = info.get_image_id().unwrap_or(-1);
                    elem.update(|elem| {
                        elem.set_atlas_id(atlas_id);
                        elem.set_image_id(image_id);
                    });
                };
                vert.write_untracked(|v| v.set_id(idx as _));
                transforms.push(*mat);
                elements.push(elem.get_untracked());
                mesh.extend_from_slice(&vert.get_untracked());
            });

            renderer.submit_data_batched(&elements, &transforms, &mesh);
        });
    }
}
