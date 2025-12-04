use std::num::NonZeroUsize;
use std::any::TypeId;

use aplite_storage::{
    Entity,
    EntityManager,
    Ptr,
    SparseSet,
    Tree,
    TypeIdMap,
    UntypedSparseSet
};

use crate::widget::{ParentWidget, Widget};

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
}

pub struct View<'a>(Box<dyn FnOnce(&mut ViewStorage) -> Entity + 'a>);

impl<'a> View<'a> {
    pub fn new<IV: IntoView>(widget: IV) -> Self {
        Self(Box::new(|cx| widget.build(cx)))
    }

    pub(crate) fn build(self, cx: &mut ViewStorage) -> Entity {
        (self.0)(cx)
    }
}

impl<'a> std::fmt::Debug for View<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.0.as_ref() as *const dyn FnOnce(&mut ViewStorage) -> Entity;
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

/// Types that automatically implement IntoView are:
/// - any type that implement Widget (`impl Widget for T`),
/// - any function that produce IntoView (`FnOnce() -> IV where IV: IntoView` or `fn() -> impl IntoView`)
pub trait IntoView: Widget + Sized + 'static {
    /// View basically is just a build context for the widget which implements it.
    /// Internally it's a `Box<dyn FnOnce(&mut ViewStorage) -> Entity + 'a>`
    fn into_view<'a>(self) -> View<'a>;
}


impl<T> IntoView for T where T: Widget + Sized + 'static {
    fn into_view<'a>(self) -> View<'a> {
        View::new(self)
    }
}

// impl<F, IV> IntoView for F
// where
//     F: FnOnce() -> IV + 'static,
//     IV: IntoView,
// {
//     fn into_view<'a>(self) -> View<'a> {
//         self().into_view()
//     }
// }
