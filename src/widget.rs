use std::cell::RefCell;
use std::collections::HashMap;

use aplite_renderer::{Shape, Scene};
use aplite_types::{Rgba, CornerRadius, Size, Rect, Unit, Paint, Matrix3x2};
use aplite_storage::{Entity, EntityManager, ArenaItem, create_entity};

use crate::layout::*;
use crate::view::{IntoView, View};
use crate::state::Border;

mod button;
mod image;
mod stack;

pub use {
    button::*,
    image::*,
    stack::*,
};

thread_local! {
    static ID_MANAGER: RefCell<EntityManager<WidgetId>> = RefCell::new(EntityManager::default());
}

create_entity! {
    pub WidgetId
}

impl WidgetId {
    pub fn new_id() -> Self {
        ID_MANAGER.with_borrow_mut(|manager| manager.create())
    }
}

pub struct Interactivity {}

/// main building block to create a renderable component
pub trait Widget {
    fn id(&self) -> &WidgetId;

    fn layout(&self, cx: &mut Layout);

    fn draw(&self, scene: &mut Scene);
}

// TODO: is immediately calculate the size here a good idea?
pub trait WidgetExt: Widget + Sized {
    fn on<F>(self, event: WidgetEvent, f: F) -> Self
    where
        F: FnMut() + 'static,
    {
        // CALLBACKS.with(|cell| {
        //     let mut storage = cell.borrow_mut();
        //     let callbacks = storage.entry(self.id()).or_default();
        //     callbacks.insert(event, Box::new(f));
        // });

        self
    }

    fn color(self, color: Rgba) -> Self {
        let _ = color;
        self
    }

    fn border_color(self, color: Rgba) -> Self {
        let _ = color;
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
        self
    }

    fn corner_radius(self, corner_radius: CornerRadius) -> Self {
        let _ = corner_radius;
        self
    }

    fn shape(mut self, shape: Shape) -> Self {
        let _ = shape;
        self
    }

    fn size(self, size: impl Into<Size>) -> Self {
        let _ = size;
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
        self
    }

    fn min_width(self, val: f32) -> Self {
        let _ = val;
        self
    }

    fn min_height(self, val: f32) -> Self {
        let _ = val;
        self
    }

    fn max_width(self, val: f32) -> Self {
        let _ = val;
        self
    }

    fn max_height(self, val: f32) -> Self {
        let _ = val;
        self
    }
}

impl<T> WidgetExt for T where T: Widget + Sized {}

impl Widget for Box<dyn Widget> {
    fn id(&self) -> &WidgetId {
        self.as_ref().id()
    }

    fn layout(&self, cx: &mut Layout) {
        self.as_ref().layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self.as_ref().draw(scene);
    }
}

impl Widget for ArenaItem<dyn Widget> {
    fn id(&self) -> &WidgetId {
        self.as_ref().id()
    }

    fn layout(&self, cx: &mut Layout) {
        self.as_ref().layout(cx);
    }

    fn draw(&self, scene: &mut Scene) {
        self.as_ref().draw(scene);
    }
}

impl std::fmt::Debug for dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
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

pub(crate) fn window(size: Size) -> WindowWidget {
    WindowWidget::new(size)
}

pub(crate) struct WindowWidget {
    id: WidgetId,
    size: Size,
    layout_rules: LayoutRules,
    children: Vec<Box<dyn IntoView>>,
}

impl WindowWidget {
    pub(crate) fn new(size: Size) -> Self {
        let layout_rules = LayoutRules::default();
        Self {
            id: WidgetId::new_id(),
            size,
            layout_rules,
            children: Vec::new(),
        }
    }
}

impl Widget for WindowWidget {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn layout(&self, cx: &mut Layout) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl IntoView for WindowWidget {
    fn into_view(self) -> View {
        View::new(self)
    }
}

// -------------------------------------

pub fn circle() -> CircleWidget {
    CircleWidget::new()
}

pub struct CircleWidget {
    id: WidgetId,
    radius: Unit,
    background: Rgba,
    border: Rgba,
    border_width: f32,
    transform: Matrix3x2,
}

impl CircleWidget {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new_id(),
            radius: Unit::Fixed(100.),
            background: Rgba::RED,
            border: Rgba::RED,
            border_width: 0.0,
            transform: Matrix3x2::identity(),
        }
    }

    pub fn radius(self, radius: Unit) -> Self {
        Self {
            radius,
            ..self
        }
    }

    pub fn background(self, color: Rgba) -> Self {
        Self {
            background: color,
            ..self
        }
    }
}

impl Widget for CircleWidget {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn layout(&self, cx: &mut Layout) {
        let parent_bound = cx.parent_rect(&self.id).map(|r| (r.width, r.height));
        let rect = cx.rects.get_or_insert(&self.id, || Rect::default());

        match self.radius {
            Unit::Fixed(r) => {
                rect.width = r;
                rect.height = r;
            },
            Unit::Grow => {
                let grow = if let Some((w, h)) = parent_bound {
                    w.min(h)
                } else {
                    cx.window_rect.width.min(cx.window_rect.height)
                };
                rect.width = grow;
                rect.height = grow;
            },
            Unit::Fit => {},
        }
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl IntoView for CircleWidget {
    fn into_view(self) -> View {
        View::new(self)
    }
}
