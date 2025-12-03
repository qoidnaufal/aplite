use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;

use aplite_renderer::Scene;
use aplite_types::{Rgba, Size, Unit};
use aplite_storage::Entity;

use crate::layout::*;
use crate::view::IntoView;
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
pub trait Widget: 'static {
    fn build(self, cx: &mut Context) -> Entity;

    fn layout(&mut self, cx: &mut Context);

    fn draw(&self, scene: &mut Scene);
}

// impl Widget for Ptr<dyn Widget> {
//     fn layout(&mut self, cx: &mut Context) {
//         self.as_mut().layout(cx);
//     }

//     fn draw(&self, scene: &mut Scene) {
//         self.as_ref().draw(scene);
//     }
// }

impl std::fmt::Debug for dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .finish()
    }
}

// -------------------------------------

pub trait ParentWidget: Widget + Sized {
    fn with_child<'a, IV: IntoView>(self, child: IV) -> Container<'a, Self> {
        Container {
            widget: self,
            children: Children::new(child),
        }
    }
}

#[derive(Debug)]
pub struct Container<'a, IV: IntoView> {
    widget: IV,
    children: Children<'a>,
}

impl<'a, IV: IntoView> Container<'a, IV> {
    pub fn with_child<C: IntoView>(mut self, child: C) -> Self {
        unsafe {
            let mut current = &mut self.children;
            while let Some(curr) = current.next.0 {
                current = &mut *curr.as_ptr();
            }

            let next = Children {
                inner: child.into_view(),
                prev: RawNode::from_raw(current),
                next: RawNode::null(),
            };

            current.next.set(next);
        }

        self
    }
}

struct RawNode<'a>(Option<NonNull<Children<'a>>>);

impl<'a> Drop for RawNode<'a> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.0.take();
            if let Some(ptr) = inner {
                ptr.drop_in_place();
            }
        }
    }
}

impl<'a> RawNode<'a> {
    pub(crate) fn null() -> Self {
        Self(None)
    }

    pub(crate) fn from_raw(raw: *mut Children<'a>) -> Self {
        unsafe {
            Self(Some(NonNull::new_unchecked(raw)))
        }
    }

    pub(crate) fn set(&mut self, next: Children<'a>) {
        let raw = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(next))) };
        self.0 = Some(raw);
    }
}

impl<'a> std::fmt::Debug for RawNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(ptr) => unsafe {
                ptr.as_ref().fmt(f)
            },
            None => write!(f, "None"),
        }
    }
}

pub struct Children<'a> {
    inner: View<'a>,
    prev: RawNode<'a>,
    next: RawNode<'a>,
}

impl<'a> std::fmt::Debug for Children<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Children")
            .field("view", &self.inner)
            .field("next", &self.next)
            .finish()
    }
}

impl<'a> Children<'a> {
    fn new<IV: IntoView>(widget: IV) -> Self {
        Self {
            inner: widget.into_view(),
            prev: RawNode::null(),
            next: RawNode::null(),
        }
    }
}

impl<IV: IntoView> Widget for Container<'static, IV> {
    fn build(self, cx: &mut Context) -> Entity {
        let id = self.widget.build(cx);
        let prev = cx.storage.set_root_id(Some(id));

        self.children.inner.build(cx);

        let mut current = self.children.next;
        while let Some(next) = current.0.take() {
            let boxed = unsafe { Box::from_raw(next.as_ptr()) };
            boxed.inner.build(cx);
            current = boxed.next;
        }

        cx.storage.set_root_id(prev);

        id
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

#[cfg(test)]
mod parent_widget_test {
    use super::*;
    use crate::view::ViewStorage;
    
    #[test]
    fn adding_child() {
        let stack = h_stack();
        let b1 = button();
        let c2 = circle();
        let w3 = window(Size::new(30., 40.));
        let c = stack.with_child(b1).with_child(c2).with_child(w3);
        println!("{c:#?}")
    }
}

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

    pub(crate) fn with_child<IV: IntoView + 'static>(self, child: IV) -> Self {
        self
    }
}

impl Widget for WindowWidget {
    fn build(self, cx: &mut Context) -> Entity {
        cx.storage.mount(self)
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

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
    fn build(self, cx: &mut Context) -> Entity {
        cx.storage.mount(self)
    }

    fn layout(&mut self, cx: &mut Context) {
       todo!() 
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}
