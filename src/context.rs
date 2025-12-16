use std::any::TypeId;
use std::num::NonZeroUsize;

use aplite_reactive::*;
use aplite_renderer::{Renderer};
use aplite_types::{Vec2f, Rect, Size};
use aplite_storage::{Entity, EntityManager, SparseSet, Tree, TypeIdMap, UntypedSparseSet};

use crate::view::{IntoView, View, AnyView};
use crate::cursor::{Cursor, MouseAction, MouseButton};
use crate::widget::Widget;
use crate::callback::CallbackStorage;

pub struct Context {
    pub(crate) current: Option<Entity>,
    pub(crate) arena: TypeIdMap<UntypedSparseSet>,
    pub(crate) id_manager: EntityManager,
    pub(crate) views: SparseSet<AnyView>,
    pub(crate) tree: Tree,
    pub(crate) type_ids: SparseSet<TypeId>,
    pub(crate) callbacks: CallbackStorage,
    pub(crate) dirty: Signal<bool>,
    pub(crate) cursor: Cursor,
    pub(crate) rect: Rect,
    pub(crate) pending_update: Vec<Entity>,
}

// ########################################################
// #                                                      #
// #                        Data                          #
// #                                                      #
// ########################################################

impl Context {
    pub(crate) fn new(size: Size, allocation_size: NonZeroUsize) -> Self {
        Self {
            current: None,
            arena: TypeIdMap::new(),
            views: SparseSet::default(),
            id_manager: EntityManager::default(),
            tree: Tree::default(),
            type_ids: SparseSet::with_capacity(allocation_size.get()),
            callbacks: CallbackStorage::default(),
            cursor: Cursor::default(),
            dirty: Signal::new(false),
            rect: Rect::from_size(size),
            pending_update: Vec::new(),
        }
    }

    pub fn set_root_id(&mut self, id: Option<Entity>) -> Option<Entity> {
        let prev = self.current.take();
        self.current = id;
        prev
    }

    pub fn mount<IV: IntoView + 'static>(&mut self, widget: IV) -> Entity {
        let type_id = TypeId::of::<IV>();
        let entity = self.id_manager.create();
        let sparse_set = self.arena
            .entry(type_id)
            .or_insert(UntypedSparseSet::new::<IV>());

        let ptr = sparse_set.insert(entity, widget).map(|iv| iv as &mut dyn Widget);
        self.views.insert(entity.id(), AnyView::new(ptr));
        self.type_ids.insert(entity.id(), type_id);
        self.tree.insert(entity.id(), self.current.as_ref().map(Entity::id));

        entity
    }

    pub(crate) fn get<IV: IntoView>(&self, entity: Entity) -> Option<&IV> {
        self.arena
            .get(&TypeId::of::<IV>())
            .and_then(|sparse_set| sparse_set.get(entity))
    }

    pub(crate) fn get_mut<IV: IntoView>(&mut self, entity: Entity) -> Option<&mut IV> {
        self.arena
            .get_mut(&TypeId::of::<IV>())
            .and_then(|sparse_set| sparse_set.get_mut(entity))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &dyn Widget> {
        self.tree
            .iter_depth(self.current.as_ref().map(|entity| entity.id()).unwrap())
            .filter_map(|entity_id| {
                self.views
                    .get(entity_id)
                    .map(|any_view| any_view.as_ref())
            })
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Widget> {
        self.tree
            .iter_depth(self.current.map(|entity| entity.id()).unwrap())
            .filter_map(|entity_id| {
                self.views
                    .get_raw(entity_id)
                    .map(|any_view| unsafe { (&mut *any_view).as_mut() })
            })
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
        self.views.iter().for_each(|(_, any)| any.as_ref().draw(&mut scene));
    }
}
