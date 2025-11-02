use std::rc::Rc;
use std::cell::RefCell;

use aplite_reactive::*;
use aplite_renderer::{Renderer};
use aplite_types::{Vec2f, Rect, Size};
use aplite_storage::EntityId;

use crate::view::{IntoView, View, ViewStorage};
use crate::cursor::{Cursor, MouseAction, MouseButton};

pub struct Context {
    pub(crate) storage: Rc<RefCell<ViewStorage>>,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pub(crate) current: Option<EntityId>,
    pub(crate) rect: Rect,
    pub(crate) pending_update: Vec<EntityId>,
}

// ########################################################
// #                                                      #
// #                        Data                          #
// #                                                      #
// ########################################################

impl Context {
    pub(crate) fn new(size: Size, allocation_size: Option<usize>) -> Self {
        Self {
            storage: Rc::new(RefCell::new(ViewStorage::new(allocation_size))),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            current: None,
            rect: Rect::from_size(size),
            pending_update: Vec::new(),
        }
    }

    pub fn spawn<IV: IntoView + 'static>(&mut self, widget: IV) -> View<IV> {
        let storage = &mut *self.storage.borrow_mut();
        let id = storage.insert(widget);
        View::new::<IV>(id, Rc::downgrade(&self.storage))
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

    fn detect_hover(&mut self) {}

    pub(crate) fn handle_drag(&mut self) {}

    pub(crate) fn handle_click(
        &mut self,
        action: impl Into<MouseAction>,
        button: impl Into<MouseButton>
    ) {
        // match self.cursor.process_click_event(action.into(), button.into()) {
        //     EmittedClickEvent::Captured(captured) => {
        //         let pos = self.state.common.query::<&Rect>().get(&captured).unwrap().vec2f();
        //         self.cursor.click.offset = self.cursor.click.pos - pos;
        //     },
        //     EmittedClickEvent::TriggerCallback(id) => {
        //         CALLBACKS.with_borrow_mut(|cb| {
        //             if let Some(callbacks) = cb.get_mut(&id)
        //                 && let MouseButton::Left = self.cursor.state.button
        //                 && let Some(callback) = callbacks.get_mut(WidgetEvent::LeftClick)
        //             {
        //                 callback();
        //                 self.toggle_dirty();
        //             }
        //         });
        //     },
        //     _ => {}
        // }
    }

    pub(crate) fn calculate_layout(&mut self) {}

// #########################################################
// #                                                       #
// #                         Render                        #
// #                                                       #
// #########################################################

    pub(crate) fn render(&self, renderer: &mut Renderer) {
        let mut scene = renderer.scene();
        let storage = &*self.storage.borrow();
        storage.views.iter().for_each(|(_, any)| any.as_ref().draw(&mut scene));
    }
}
