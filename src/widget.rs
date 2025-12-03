use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::Scene;
use aplite_types::{Rgba, Size, Unit};
use aplite_storage::Entity;

use crate::layout::*;
use crate::view::{IntoView, ViewStorage};
use crate::context::Context;
use crate::view::View;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

/// main building block to create a renderable component
pub trait Widget {
    fn build(self, cx: &mut ViewStorage) -> Entity;

    fn layout(&mut self, cx: &mut Context);

    fn draw(&self, scene: &mut Scene);
}

impl<F, IV> Widget for F
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView,
{
    fn build(self, storage: &mut ViewStorage) -> Entity {
        self().into_view().build(storage)
    }

    fn layout(&mut self, _: &mut Context) {}

    fn draw(&self, _: &mut aplite_renderer::Scene) {}
}

// -------------------------------------

pub trait ParentWidget: IntoView {
    fn with_child<'a, IV: IntoView>(self, child: IV) -> Container<'a, Self> {
        Container {
            widget: self,
            children: Children::new(child),
        }
    }
}

pub struct Container<'a, IV: IntoView> {
    widget: IV,
    children: Children<'a>,
}

impl<'a, IV: IntoView> Container<'a, IV> {
    pub fn with_child<C: IntoView>(mut self, child: C) -> Self {
        self.children.0.push(child.into_view());
        self
    }
}

impl<IV: IntoView> Widget for Container<'static, IV> {
    fn build(mut self, cx: &mut ViewStorage) -> Entity {
        let id = self.widget.into_view().build(cx);
        let prev = cx.set_root_id(Some(id));

        let mut children = std::mem::take(&mut self.children.0);
        children.drain(..)
            .for_each(|child| {
                child.build(cx);
            });

        cx.set_root_id(prev);

        id
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl<'a, IV: IntoView> std::fmt::Debug for Container<'a, IV> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field("widget", &std::any::type_name_of_val(&self.widget))
            .field("children", &self.children)
            .finish()
    }
}

pub struct Children<'a>(Vec<View<'a>>);

impl<'a> std::fmt::Debug for Children<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(&self.0)
            .finish()
    }
}

impl<'a> Children<'a> {
    fn new<IV: IntoView>(widget: IV) -> Self {
        Self(vec![widget.into_view()])
    }
}

// impl<IV: IntoView> IntoView for Container<'static, IV> {
//     fn into_view<'a>(self) -> View<'a> {
//         View::new(self)
//     }
// }

// -------------------------------------

pub trait InteractiveWidget: Widget {}
//     fn on(&self, event: WidgetEvent, f: impl FnMut() + 'static) {
//         CALLBACKS.with(|cell| {
//             let mut storage = cell.borrow_mut();
//             let callbacks = storage.entry(self.entity).or_default();
//             callbacks.insert(event, Box::new(f));
//         });
//     }

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
pub(crate) struct CallbackStore([Option<Box<dyn FnMut()>>; 5]);

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

// -------------------------------------

pub(crate) fn window(size: Size) -> WindowWidget {
    WindowWidget::new(size)
}

pub(crate) struct WindowWidget {
    size: Size,
    layout_rules: LayoutRules,
}

impl WindowWidget {
    pub(crate) fn new(size: Size) -> Self {
        let layout_rules = LayoutRules::default();
        Self {
            size,
            layout_rules,
        }
    }
}

impl Widget for WindowWidget {
    fn build(self, cx: &mut ViewStorage) -> Entity {
        cx.mount(self)
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

// impl IntoView for WindowWidget {
//     fn into_view<'a>(self) -> View<'a> {
//         View::new(self)
//     }
// }

impl ParentWidget for WindowWidget {}

// -------------------------------------

pub fn circle() -> CircleWidget {
    CircleWidget::new()
}

pub struct CircleWidget {
    radius: Unit,
}

impl CircleWidget {
    pub fn new() -> Self {
        Self {
            radius: Unit::Fixed(100.),
        }
    }

    pub fn radius(self, radius: Unit) -> Self {
        Self {
            radius,
            ..self
        }
    }
}

impl Widget for CircleWidget {
    fn build(self, cx: &mut ViewStorage) -> Entity {
        cx.mount(self)
    }

    fn layout(&mut self, cx: &mut Context) {
       todo!() 
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

// impl IntoView for CircleWidget {
//     fn into_view<'a>(self) -> View<'a> {
//         View::new(self)
//     }
// }

#[cfg(test)]
mod parent_widget_test {
    use super::*;

    fn abc() -> impl IntoView {
        let stack = h_stack();
        let b1 = button();
        let c2 = circle();
        let w3 = window(Size::new(30., 40.));
        stack.with_child(b1).with_child(c2).with_child(w3)
    }
    
    #[test]
    fn adding_child() {
        let abc = abc();
        let container = &abc as *const dyn Widget as *const Container<'static, HStack>;
        unsafe {
            println!("{:#?}", &*container)
        }
    }
}
