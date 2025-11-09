use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Weak;

use aplite_storage::{Arena, ArenaItem, SparseSet, Entity, EntityManager, Tree};

use crate::widget::{InteractiveWidget, ParentWidget, Widget};

pub(crate) struct ViewStorage {
    pub(crate) arena: Arena,
    pub(crate) id_manager: EntityManager,
    pub(crate) views: SparseSet<AnyView>,
    pub(crate) tree: Tree,
}

impl ViewStorage {
    pub(crate) fn new(allocation_size: Option<usize>) -> Self {
        let allocation_size = allocation_size.unwrap_or(1024 * 1024);
        Self {
            arena: Arena::new(allocation_size),
            views: SparseSet::default(),
            id_manager: EntityManager::default(),
            tree: Tree::default(),
        }
    }

    pub(crate) fn insert<IV: IntoView + 'static>(&mut self, widget: IV) -> Entity {
        let item = self.arena.alloc_mapped(widget.into_view(), |w| w as &mut dyn Widget);
        let id = self.id_manager.create();
        self.views.insert(&id, AnyView::new(item));
        id
    }
}

pub trait IntoView {
    type View: Widget;
    fn into_view(self) -> Self::View;
}

pub struct View<IV: IntoView> {
    id: Entity,
    cx: Weak<RefCell<ViewStorage>>,
    marker: PhantomData<IV>,
}

impl<IV: IntoView> Clone for View<IV> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            cx: Weak::clone(&self.cx),
            marker: PhantomData,
        }
    }
}

impl<IV: IntoView + 'static> View<IV> {
    pub(crate) fn new<T: IntoView + 'static>(id: Entity, cx: Weak<RefCell<ViewStorage>>) -> Self {
        Self {
            id,
            cx,
            marker: PhantomData,
        }
    }

    pub(crate) fn id(&self) -> &Entity {
        &self.id
    }

    pub(crate) fn with_storage_ref<R>(&self, f: impl FnOnce(&ViewStorage) -> R) -> R {
        let rc = self.cx.upgrade().unwrap();
        f(&*rc.borrow())
    }

    pub(crate) fn with_storage_mut<R>(&self, f: impl FnOnce(&mut ViewStorage) -> R) -> R {
        let rc = self.cx.upgrade().unwrap();
        f(&mut *rc.borrow_mut())
    }

    pub fn with<R>(&self, f: impl FnOnce(&IV::View) -> R) -> R {
        self.with_storage_ref(|s| {
            let anyview = s.views.get(&self.id).unwrap();
            f(anyview.downcast_ref::<IV>())
        })
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut IV::View) -> R) -> R {
        self.with_storage_mut(|s| {
            let anyview = s.views.get_mut(&self.id).unwrap();
            f(anyview.downcast_mut::<IV>())
        })
    }
}

impl<IV> View<IV>
where
    IV: IntoView + 'static,
    IV::View: ParentWidget,
{
    pub fn child<W: IntoView + 'static>(self, child: &View<W>) -> Self {
        self.with_storage_mut(|s| {
            s.tree.insert(child.id, Some(self.id));
        });
        self
    }

    pub fn map_children<R>(&self, mut f: impl FnMut(&dyn Widget) -> R + 'static) -> Vec<R> {
        self.with_storage_ref(|s| {
            s.tree
                .iter_children(&self.id)
                .filter_map(move |id| {
                    s.views.get(id).map(|any| f(any.as_ref()))
                })
                .collect()
        })
    }

    pub fn for_each_child(&self, mut f: impl FnMut(&dyn Widget) + 'static) {
        self.with_storage_ref(|s| {
            s.tree
                .iter_children(&self.id)
                .for_each(move |id| {
                    if let Some(child) = s.views.get(id) {
                        f(child.as_ref())
                    }
                })
        })
    }
}

thread_local! {
    pub(crate) static CALLBACKS: RefCell<HashMap<Entity, CallbackStore>>
        = RefCell::new(Default::default());
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetEvent {
    Hover,
    LeftClick,
    RightClick,
    Drag,
    Input,
}

#[derive(Default)]
pub(crate) struct CallbackStore(Box<[Option<Box<dyn FnMut()>>; 5]>);

impl CallbackStore {
    pub(crate) fn insert(
        &mut self,
        event: WidgetEvent,
        callback: Box<dyn FnMut()>,
    ) {
        self.0[event as usize].replace(callback);
    }

    pub(crate) fn get_mut(&mut self, event: WidgetEvent) -> Option<&mut Box<dyn FnMut()>> {
        self.0[event as usize].as_mut()
    }
}

impl<IV> View<IV>
where
    IV: IntoView + 'static,
    IV::View: InteractiveWidget,
{
    pub fn on(&self, event: WidgetEvent, f: impl FnMut() + 'static) {
        CALLBACKS.with(|cell| {
            let mut storage = cell.borrow_mut();
            let callbacks = storage.entry(self.id).or_default();
            callbacks.insert(event, Box::new(f));
        });
    }
}

pub(crate) struct AnyView {
    pub(crate) item: ArenaItem<dyn Widget>,
}

impl AnyView {
    pub(crate) fn new(item: ArenaItem<dyn Widget>) -> Self {
        Self { item }
    }

    pub(crate) fn as_ref(&self) -> &dyn Widget {
        &self.item
    }

    pub(crate) fn as_mut(&mut self) -> &mut dyn Widget {
        &mut self.item
    }

    pub(crate) fn downcast_ref<IV: IntoView>(&self) -> &IV::View {
        unsafe {
            let raw = self.item.as_ref() as *const dyn Widget as *const IV::View;
            &*raw
        }
    }

    pub(crate) fn downcast_mut<IV: IntoView>(&mut self) -> &mut IV::View {
        unsafe {
            let raw = self.item.as_mut() as *mut dyn Widget as *mut IV::View;
            &mut *raw
        }
    }
}
