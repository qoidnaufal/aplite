use std::num::NonZeroUsize;
use std::any::TypeId;

use aplite_storage::{
    Arena,
    CpuBuffer,
    Entity,
    EntityManager,
    Ptr,
    SparseSet,
    Tree,
    TypeIdMap,
    UntypedSparseSet
};

use crate::widget::{ParentWidget, Widget};
use crate::context::Context;

pub struct ViewStorage {
    pub(crate) current: Option<Entity>,
    pub(crate) arena: TypeIdMap<UntypedSparseSet>,
    pub(crate) id_manager: EntityManager,
    pub(crate) views: SparseSet<AnyView>,
    pub(crate) tree: Tree,
    type_ids: SparseSet<TypeId>,
}

impl ViewStorage {
    pub(crate) fn new(allocation_size: NonZeroUsize) -> Self {
        Self {
            current: None,
            arena: TypeIdMap::new(),
            views: SparseSet::default(),
            id_manager: EntityManager::default(),
            tree: Tree::default(),
            type_ids: SparseSet::with_capacity(allocation_size.get()),
        }
    }

    pub fn set_root_id(&mut self, id: Option<Entity>) -> Option<Entity> {
        let prev = self.current.take();
        self.current = id;
        prev
    }

    pub(crate) fn mount<IV: IntoView + 'static>(&mut self, widget: IV) -> Entity {
        let type_id = TypeId::of::<IV>();
        let entity = self.id_manager.create();
        let sparse_set = self.arena
            .entry(type_id)
            .or_insert(UntypedSparseSet::new::<IV>());

        // let ptr = self.arena.alloc_mapped(widget, |w| w as &mut dyn Widget);

        // self.views.insert(entity.id(), AnyView::new(ptr));
        sparse_set.insert(&entity, widget);
        self.type_ids.insert(entity.id(), type_id);
        self.tree.insert(*entity.id(), self.current.as_ref().map(Entity::id));

        entity
    }
}

pub trait IntoView: Widget + Sized {
    fn into_view<'a>(self) -> View<'a>;
}

impl<T> IntoView for T where T: Widget {
    fn into_view<'a>(self) -> View<'a> {
        View::new(self)
    }
}

pub struct View<'a>(Box<dyn FnOnce(&mut Context) -> Entity + 'a>);

impl<'a> View<'a> {
    fn new<IV: IntoView>(widget: IV) -> Self {
        Self(Box::new(|cx| widget.build(cx)))
    }

    pub(crate) fn build(self, cx: &mut Context) -> Entity {
        (self.0)(cx)
    }
}

impl<'a> std::fmt::Debug for View<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.0.as_ref() as *const dyn FnOnce(&mut Context) -> Entity;
        f.debug_struct("View")
            .field("addr", &inner.addr())
            .finish()
    }
}

pub(crate) struct AnyView {
    pub(crate) ptr: Ptr<dyn Widget>,
}

impl AnyView {
    pub(crate) fn new(item: Ptr<dyn Widget>) -> Self {
        Self { ptr: item }
    }

    pub(crate) fn as_ref(&self) -> &dyn Widget {
        self.ptr.as_ref()
    }

    pub(crate) fn as_mut(&mut self) -> &mut dyn Widget {
        self.ptr.as_mut()
    }

    // pub(crate) fn downcast_ref<IV: IntoView>(&self) -> &IV::View {
    //     unsafe {
    //         let raw = self.ptr.as_ref() as *const dyn Widget as *const IV::View;
    //         &*raw
    //     }
    // }

    // pub(crate) fn downcast_mut<IV: IntoView>(&mut self) -> &mut IV::View {
    //     unsafe {
    //         let raw = self.ptr.as_mut() as *mut dyn Widget as *mut IV::View;
    //         &mut *raw
    //     }
    // }
}

impl std::ops::Deref for AnyView {
    type Target = dyn Widget;

    fn deref(&self) -> &Self::Target {
        std::ops::Deref::deref(&self.ptr)
    }
}

impl std::ops::DerefMut for AnyView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        std::ops::DerefMut::deref_mut(&mut self.ptr)
    }
}
