use aplite_reactive::*;
use aplite_renderer::{Renderer};
use aplite_types::{Vec2f, Rect};
use aplite_storage::{Entity, EntityManager};

use crate::view::View;
use crate::widget::{CALLBACKS, WidgetId, WidgetEvent};
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::Layout;
use crate::state::{State, Flag};

pub struct Context {
    pub(crate) view: View,
    pub(crate) manager: EntityManager<WidgetId>,
    pub(crate) state: State,
    pub(crate) layout: Layout,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pub(crate) current: Option<WidgetId>,
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
            manager: EntityManager::default(),
            state: State::default(),
            layout: Layout::new(),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            current: None,
            pending_update: Vec::new(),
        }
    }

    pub fn create_id(&mut self) -> WidgetId {
        let id = self.manager.create();
        self.state.insert_default_state(&id);
        id
    }

    pub(crate) fn toggle_dirty(&self) {
        self.dirty.set(true);
    }

    pub(crate) fn process_pending_update(&mut self) {
        // NODE_STORAGE.with_borrow(|s| {
        //     s.iter()
        //         .filter_map(|(id, state)| {
        //             let id = state.borrow().flag.needs_relayout.then_some(id);
        //             state.borrow_mut().flag.needs_relayout = false;
        //             id
        //         })
        //         .for_each(|id| self.pending_update.push(*id));
        // });

        if !self.pending_update.is_empty() {
            self.pending_update
                .drain(..)
                .for_each(|_id| {
                    // if let Some(widget) = self.view.find_parent(&id)
                    //     && let Some(children) = widget.children_ref()
                    // {
                    //     widget.calculate_size(None);
                    //     let mut cx = LayoutCx::new(widget.as_ref());
                    //     children.iter()
                    //         .for_each(|child| child.calculate_layout(&mut cx));
                    // }
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
        if let Some(id) = self.cursor.hover.curr {
            let rect = self.state.common.fetch_one::<&Rect>(&id).unwrap();
            let contains = rect.contains(self.cursor.hover_pos());

            if !contains {
                self.cursor.hover.curr = self.layout.tree.get_parent(&id).copied();
            } else {
                self.cursor.hover.curr = self.layout.tree
                    .iter_children(&id)
                    .find(|child| {
                        let child_rect = self.state.common.fetch_one::<&Rect>(*child).unwrap();
                        child_rect.contains(self.cursor.hover_pos())
                    })
                    .or(Some(&id))
                    .copied();
            }
        } else {
            self.cursor.hover.curr = self.layout.tree
                .iter_depth(&WidgetId::root())
                .filter(|member| {
                    self.state.common
                        .fetch_one::<&Flag>(member)
                        .is_some_and(|flag| flag.visible)
                })
                .find(|member| {
                    let rect = self.state.common.fetch_one::<&Rect>(*member).unwrap();
                    rect.contains(self.cursor.hover_pos())
                })
                .copied();
        }

        // let hovered = self.view.detect_hover(&self.cursor).map(Widget::id);
        // self.cursor.hover.curr = hovered;
    }

    pub(crate) fn handle_drag(&mut self) {
        if let Some(captured) = self.cursor.click.captured {
            let (rect, flag) = self.state.common
                .fetch::<(&mut Rect, &mut Flag)>(&captured)
                .unwrap();

            if self.cursor.is_dragging() && flag.movable {
                self.cursor.is_dragging = true;
                let pos = self.cursor.hover.pos - self.cursor.click.offset;

                rect.set_pos(pos);
                flag.needs_relayout = true;
                flag.needs_redraw = true;

                self.layout.calculate_position(&captured, &self.state);
                // TODO: dirty scissor rect
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
            EmittedClickEvent::Captured(captured) => {
                let pos = self.state.common.fetch_one::<&Rect>(&captured).unwrap().vec2f();
                self.cursor.click.offset = self.cursor.click.pos - pos;
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

    pub(crate) fn calculate_layout(&mut self) {
        self.layout.calculate_layout(&WidgetId::root(), &self.state);
    }

// #########################################################
// #                                                       #
// #                         Render                        #
// #                                                       #
// #########################################################

    pub(crate) fn render(&self, renderer: &mut Renderer) {
        self.state.render(renderer, &self.layout.tree);

        // let mut scene = renderer.scene();
        // self.view.widget.draw(&mut scene);
    }
}
