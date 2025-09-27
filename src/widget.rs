use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;

use aplite_renderer::{Shape, Scene};
use aplite_types::{Rgba, CornerRadius, Size, Rect};
use aplite_storage::{Tree, EntityManager, Entity, create_entity};

use crate::context::Context;
use crate::layout::*;
use crate::view::IntoView;
use crate::state::{
    ViewNode,
    NodeRef,
    AspectRatio,
};

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

thread_local! {
    pub(crate) static ENTITY_MANAGER: RefCell<EntityManager<WidgetId>> =
        RefCell::new(EntityManager::default());
}

create_entity! {
    pub WidgetId
}

/// main building block to create a renderable component
pub trait Widget {
    fn id(&self) -> WidgetId;

    fn node(&self) -> ViewNode;

    fn children(&self) -> Option<&Children> {
        None
    }

    // fn draw(&self, scene: &mut Scene) {
    //     let node = self.node_ref().unwrap().upgrade();

    //     if !node.borrow().flag.is_hidden() {
    //         if node.borrow().flag.is_dirty() {
    //             let state = node.borrow();

    //             scene.draw(&aplite_renderer::DrawArgs {
    //                 rect: &state.rect,
    //                 transform: &state.transform,
    //                 background_paint: &state.background_paint.as_paint_ref(),
    //                 border_paint: &state.border_paint.as_paint_ref(),
    //                 border_width: state.border_width.max(5.0),
    //                 shape: state.shape,
    //                 corner_radius: state.corner_radius,
    //             });

    //             drop(state);

    //             node.borrow_mut().flag.set_dirty(false);
    //         } else {
    //             scene.next_draw();
    //         }

    //         if let Some(children) = self.children_ref() {
    //             children
    //                 .iter()
    //                 .for_each(|child| {
    //                     child.draw(scene);
    //                 });
    //         }
    //     }
    // }

    // fn layout(&self, cx: &mut LayoutCx) -> bool {
    //     let node = self.node_ref().unwrap().upgrade();
    //     if node.borrow().flag.is_hidden() { return false }

    //     let size = node.borrow().rect.size();
    //     let mut this = node.borrow_mut();

    //     match cx.rules.orientation {
    //         Orientation::Vertical => {
    //             match cx.rules.align_h {
    //                 AlignH::Left | AlignH::Right => this.rect.x = cx.next_pos.x,
    //                 AlignH::Center => this.rect.x = cx.next_pos.x - size.width / 2.,
    //             }

    //             this.rect.y = cx.next_pos.y;
    //             cx.next_pos.y += cx.rules.spacing as f32 + size.height;
    //         },
    //         Orientation::Horizontal => {
    //             match cx.rules.align_v {
    //                 AlignV::Top | AlignV::Bottom => this.rect.y = cx.next_pos.y,
    //                 AlignV::Middle => this.rect.y = cx.next_pos.y - size.height / 2.,
    //             }

    //             this.rect.x = cx.next_pos.x;
    //             cx.next_pos.x += cx.rules.spacing as f32 + size.width;
    //         },
    //     }

    //     this.flag.set_dirty(true);

    //     true
    // }

}

pub struct Children(UnsafeCell<Vec<Box<dyn IntoView>>>);

impl std::fmt::Debug for Children {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            if let Some(vec) = self.0.get().as_ref() {
                f.debug_list()
                    .entries(vec)
                    .finish()
            } else {
                write!(f, "[]")
            }
        }
    }
}

impl Children {
    pub fn new() -> Self {
        Self(UnsafeCell::new(Vec::new()))
    }

    pub fn push(&self, child: impl IntoView + 'static) {
        unsafe {
            if let Some(vec) = self.0.get().as_mut() {
                vec.push(Box::new(child));
            }
        }
    }

    pub(crate) fn iter_all(&self) -> impl Iterator<Item = &dyn IntoView> {
        unsafe {
            self.0.get()
                .as_ref()
                .unwrap()
                .iter()
                .map(|child| child.as_ref())
        }
    }

    pub(crate) fn drain(self) -> impl Iterator<Item = Box<dyn IntoView>> {
        unsafe {
            self.0.get()
                .as_mut()
                .unwrap()
                .drain(..)
        }
    }
}

pub struct ChildrenRef<'a>(&'a [Box<dyn Widget>]);

impl<'a> ChildrenRef<'a> {
    pub fn all_ref(&self) -> impl Iterator<Item = &'a dyn Widget> {
        self.0.iter().map(|child| child.as_ref())
    }

    pub fn all_boxed(&self) -> impl Iterator<Item = &'a Box<dyn Widget>> {
        self.0.iter()
    }

    pub fn visible_ref(&self) -> impl Iterator<Item = &'a dyn Widget> {
        self.0.iter()
            .filter_map(|child| {
                child.node()
                    .is_visible()
                    .then_some(child.as_ref())
            })
    }

    pub fn visible_boxed(&self) -> impl Iterator<Item = &'a Box<dyn Widget>> {
        self.0.iter()
            .filter_map(|child| {
                child.node()
                    .is_visible()
                    .then_some(child)
            })
    }
}

impl<'a> From<&'a Vec<Box<dyn Widget>>> for ChildrenRef<'a> {
    fn from(value: &'a Vec<Box<dyn Widget>>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ChildrenRef<'_> {
    type Target = [Box<dyn Widget>];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::fmt::Debug for ChildrenRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(&self.0)
            .finish()
    }
}

pub struct ChildrenMut<'a>(&'a mut Vec<Box<dyn Widget>>);

impl<'a> From<&'a mut Vec<Box<dyn Widget>>> for ChildrenMut<'a> {
    fn from(value: &'a mut Vec<Box<dyn Widget>>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ChildrenMut<'_> {
    type Target = Vec<Box<dyn Widget>>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::ops::DerefMut for ChildrenMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl std::fmt::Debug for ChildrenMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(self.0)
            .finish()
    }
}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn child(self, child: impl IntoView + 'static) -> Self {
        if let Some(children) = self.children() {
            children.push(child);
        }
        self
    }

    fn on<F>(self, event: WidgetEvent, f: F) -> Self
    where
        F: FnMut() + 'static,
    {
        CALLBACKS.with(|cell| {
            let mut storage = cell.borrow_mut();
            let callbacks = storage.entry(self.id()).or_default();
            callbacks.insert(event, Box::new(f));
        });

        self
    }

    // fn image_aspect_ratio(self, val: AspectRatio) -> Self {
    //     self.node_ref()
    //         .unwrap()
    //         .upgrade()
    //         .borrow_mut()
    //         .image_aspect_ratio = val;

    //     self
    // }

    fn color(self, color: Rgba) -> Self {
        let _ = color;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .background_paint = color.into();

        self
    }

    fn border_color(self, color: Rgba) -> Self {
        let _ = color;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .border_paint = color.into();

        self
    }

    fn hover_color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn click_color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn border_width(self, val: f32) -> Self {
        let _ = val;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .border_width = val;

        self
    }

    fn corner_radius(self, corner_radius: CornerRadius) -> Self {
        let _ = corner_radius;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .corner_radius = corner_radius;

        self
    }

    fn shape(self, shape: Shape) -> Self {
        let _ = shape;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .shape = shape;

        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        let _ = size;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .rect.set_size(size.into());

        self
    }

    fn width(self, width: Unit) -> Self {
        let _ = width;
        self
    }

    fn height(self, height: Unit) -> Self {
        let _ = height;
        self
    }

    fn dragable(self) -> Self {
        // let node = self.node_ref().unwrap().upgrade();
        // let mut node = node.borrow_mut();
        // node.flag.set_dragable(true);
        // node.flag.set_hoverable(true);

        self
    }

    fn spacing(self, val: u8) -> Self {
        let _ = val;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .spacing = val;

        self
    }

    fn padding(self, padding: Padding) -> Self {
        let _ = padding;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .padding = padding;

        self
    }

    fn min_width(self, val: f32) -> Self {
        let _ = val;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .min_width = Some(val);

        self
    }

    fn min_height(self, val: f32) -> Self {
        let _ = val;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .min_height = Some(val);

        self
    }

    fn max_width(self, val: f32) -> Self {
        let _ = val;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .max_width = Some(val);

        self
    }

    fn max_height(self, val: f32) -> Self {
        let _ = val;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .max_height = Some(val);

        self
    }

    fn align_h(self, align_h: AlignH) -> Self {
        let _ = align_h;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .align_h = align_h;

        self
    }

    fn align_v(self, align_v: AlignV) -> Self {
        let _ = align_v;
        // self.node_ref()
        //     .unwrap()
        //     .upgrade()
        //     .borrow_mut()
        //     .align_v = align_v;

        self
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

impl Widget for Box<dyn Widget> {
    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children(&self) -> Option<&Children> {
        self.as_ref().children()
    }
}

impl Widget for Box<&mut dyn Widget> {
    fn id(&self) -> WidgetId {
        self.as_ref().id()
    }

    fn node(&self) -> ViewNode {
        self.as_ref().node()
    }

    fn children(&self) -> Option<&Children> {
        self.as_ref().children()
    }
}

impl std::fmt::Debug for Box<dyn Widget> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Debug for &dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("id", &self.id())
            .field("children", &self.children().unwrap_or(&Children::new()))
            .finish()
    }
}

// -------------------------------------

thread_local! {
    pub(crate) static CALLBACKS: RefCell<HashMap<WidgetId, CallbackStore>>
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

// -------------------------------------

pub(crate) struct WindowWidget {
    id: WidgetId,
    children: Children,
}

impl WindowWidget {
    pub(crate) fn new(cx: &mut Context, rect: Rect) -> Self {
        Self {
            id: cx.create_id(),
            children: Children::new(),
        }
    }
}

impl Widget for WindowWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn node(&self) -> ViewNode {
        todo!()
    }

    fn children(&self) -> Option<&Children> {
        Some(&self.children)
    }
}

// -------------------------------------

pub struct CircleWidget {
    id: WidgetId,
}

impl CircleWidget {
    pub fn new(cx: &mut Context) -> Self {
        let id = cx.create_id();
        // node: ViewNode::default()
        //     .with_stroke_width(5.)
        //     .with_shape(Shape::Circle)
        //     .with_size((100., 100.)),
        Self { id }
    }
}

impl Widget for CircleWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn node(&self) -> ViewNode {
        todo!()
    }
}
