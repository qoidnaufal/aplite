use std::any::TypeId;
use std::num::NonZeroUsize;

use aplite_reactive::*;
use aplite_renderer::{Renderer, Scene};
use aplite_types::{Rect, Rgba, Size, Vec2f};
use aplite_storage::{ComponentStorage, ComponentTuple, Entity, EntityId, EntityManager};

use crate::view::IntoView;
use crate::cursor::{Cursor, MouseAction, MouseButton};
use crate::widget::Widget;
use crate::callback::CallbackStorage;

pub struct Context {
    pub(crate) id_manager: EntityManager,
    pub(crate) storage: ComponentStorage,
    pub(crate) callbacks: CallbackStorage,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pub(crate) window_rect: Rect,
    pub(crate) pending_update: Vec<Entity>,
}

/*
########################################################
#
# Data
#
########################################################
*/

impl Context {
    pub(crate) fn new(size: Size, _allocation_size: NonZeroUsize) -> Self {
        Self {
            id_manager: EntityManager::default(),
            storage: ComponentStorage::new(),
            callbacks: CallbackStorage::default(),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            window_rect: Rect::from_size(size),
            pending_update: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        self.id_manager.create()
    }

    pub fn register<C: ComponentTuple>(&mut self, entity: Entity, component_tuple: C) {
        self.storage.insert_component_tuple(entity, component_tuple);
    }

    pub fn mount<IV: IntoView>(&mut self, widget: IV) {
    }

    pub fn layout(&mut self) {}

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

/*
#########################################################
#
# Cursor Event
#
#########################################################
*/

    pub(crate) fn handle_mouse_move(&mut self, pos: impl Into<Vec2f>) {
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_hover();
        #[cfg(feature = "cursor_stats")] eprint!("{:?}     \r", start.elapsed());

        self.handle_drag();
    }

    fn detect_hover(&mut self) {
        let query = self.storage.query::<(&Vec2f, &mut Size)>();
    }

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

    pub(crate) fn render<W: Widget>(&self, widget: &W, renderer: &mut Renderer) {
        let mut scene = renderer.scene();
        widget.draw(&mut scene);
    }
}

pub struct RenderCx<'a> {
    cx: &'a mut Context,
    scene: &'a mut Scene<'a>,
}

pub struct Theme {
    pub red0: Rgba,
    pub red1: Rgba,
    pub green0: Rgba,
    pub green1: Rgba,
    pub blue0: Rgba,
    pub blue1: Rgba,
    pub yellow0: Rgba,
    pub yellow1: Rgba,
    pub orange0: Rgba,
    pub orange1: Rgba,
    pub purple0: Rgba,
    pub purple1: Rgba,
    pub background0: Rgba,
    pub background1: Rgba,
    pub foreground0: Rgba,
    pub foreground1: Rgba,
}
