use std::marker::PhantomData;

use aplite_storage::{Component, ComponentRegistrator, ComponentStorage, EntityIdMap};

use crate::{context::Context, view::IntoView, widget::Widget};

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
pub(crate) struct CallbackStorage(ComponentStorage);

impl std::ops::Deref for CallbackStorage {
    type Target = ComponentStorage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CallbackStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) trait WidgetEventType {
    const EVENT: WidgetEvent;

    fn id(&self) -> usize;
}

pub(crate) struct Hover; impl WidgetEventType for Hover {
    const EVENT: WidgetEvent = WidgetEvent::Hover;
    fn id(&self) -> usize { 0 }
}

pub(crate) struct LeftClick; impl WidgetEventType for LeftClick {
    const EVENT: WidgetEvent = WidgetEvent::LeftClick;
    fn id(&self) -> usize { 1 }
}

pub(crate) struct RightClick; impl WidgetEventType for RightClick {
    const EVENT: WidgetEvent = WidgetEvent::RightClick;
    fn id(&self) -> usize { 2 }
}

pub(crate) struct Drag; impl WidgetEventType for Drag {
    const EVENT: WidgetEvent = WidgetEvent::Drag;
    fn id(&self) -> usize { 3 }
}

pub(crate) struct Input; impl WidgetEventType for Input {
    const EVENT: WidgetEvent = WidgetEvent::Input;
    fn id(&self) -> usize { 4 }
}

pub(crate) struct Callback<E: WidgetEventType> {
    f: Box<dyn FnMut()>,
    marker: PhantomData<E>
}

impl<E: WidgetEventType> Callback<E> {
    pub(crate) fn new<F: FnMut() + 'static>(f: F) -> Self {
        Self {
            f: Box::new(f),
            marker: PhantomData,
        }
    }
}

impl<E: WidgetEventType + 'static> Component for Callback<E> {}

pub trait InteractiveWidget: Widget + Sized + 'static {
    fn on<E, F>(self, cx: &mut Context, f: F) -> aplite_storage::Entity
    where
        E: WidgetEventType + 'static,
        F: FnMut() + 'static,
    {
        let registrator = cx.callbacks.registrator();
        let table_id = registrator.register::<Callback<E>>().finish();

        let entity = self.into_view().build(cx);
        // cx.callbacks.insert_with_table_id(table_id, Callback::<E>::new(f));

        entity
    }
}
