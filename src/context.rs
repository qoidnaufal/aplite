use aplite_reactive::*;
use aplite_renderer::{Renderer, DrawArgs};
use aplite_types::{Vec2f, Rect};
use aplite_storage::{EntityManager, Tree};

use crate::view::View;
use crate::widget::{CALLBACKS, Widget, WidgetId, WidgetEvent};
use crate::cursor::{Cursor, MouseAction, MouseButton, EmittedClickEvent};
use crate::layout::State;
use crate::state::NODE_STORAGE;

pub struct Context {
    pub(crate) view: View,
    pub(crate) entity_manager: EntityManager<WidgetId>,
    pub(crate) tree: Tree<WidgetId>,
    pub(crate) state: State,
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
            entity_manager: EntityManager::default(),
            tree: Tree::default(),
            state: State::new(),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            current: None,
            pending_update: Vec::new(),
        }
    }

    pub fn create_id(&mut self) -> WidgetId {
        let id = self.entity_manager.create();
        self.state.insert_default_state(&id);
        id
    }

    pub(crate) fn insert_new_id(&mut self, id: &WidgetId) {
        let current = self.current.take();
        if let Some(parent) = current.as_ref() {
            self.tree.insert(*id, parent);
        }
        self.current = current;
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
                .for_each(|id| self.pending_update.push(*id));
        });

        if !self.pending_update.is_empty() {
            self.pending_update
                .drain(..)
                .for_each(|id| {
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
        let Vec2f { x, y } = self.cursor.hover.pos;

        if let Some(id) = self.cursor.hover.curr {
            let pos = self.state.get_position(&id).unwrap();
            let size = self.state.get_size(&id).unwrap();
            let contains = (pos.x..pos.x + size.width).contains(&x) && (pos.y..pos.y + size.height).contains(&y);

            if !contains {
                self.cursor.hover.curr = self.tree.get_parent(&id).copied();
            } else {
                self.cursor.hover.curr = self.tree
                    .iter_children(&id)
                    .find(|child| {
                        let child_pos = self.state.get_position(*child).unwrap();
                        let child_size = self.state.get_size(*child).unwrap();
                        (child_pos.x..child_pos.x + child_size.width).contains(&x)
                            && (child_pos.y..child_pos.y + child_size.height).contains(&y)
                    })
                    .or(Some(&id))
                    .copied();
            }
        } else {
            self.cursor.hover.curr = self.tree
                .iter_depth(&self.view.id())
                .filter(|member| {
                    self.state
                        .get_flag(member)
                        .is_some_and(|flag| flag.is_visible())
                })
                .find(|member| {
                    let pos = self.state.get_position(*member).unwrap();
                    let size = self.state.get_size(*member).unwrap();
                    (pos.x..pos.x + size.width).contains(&x) && (pos.y..pos.y + size.height).contains(&y)
                })
                .copied();
        }

        // let hovered = self.view.detect_hover(&self.cursor).map(Widget::id);
        // self.cursor.hover.curr = hovered;
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

                    self.state.calculate_position(&self.tree, &captured);
                    // let widget = self.view.find_visible(&captured).unwrap();
                    // if let Some(children) = widget.children_ref() {
                    //     let mut cx = LayoutCx::new(widget);
                    //     children.iter()
                    //         .for_each(|child| child.calculate_layout(&mut cx));
                    // }

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
        self.state.render(renderer);

        // let mut scene = renderer.scene();
        // self.view.widget.draw(&mut scene);
    }
}
