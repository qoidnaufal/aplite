use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::HashMap;

use aplite_renderer::Shape;
use aplite_types::{Rgba, CornerRadius, Size, Rect, Unit};
use aplite_storage::{EntityManager, Entity, Tree, create_entity};

use crate::layout::*;
use crate::view::{IntoView, View};
use crate::state::WidgetState;

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

    pub(crate) static TREE: RefCell<Tree<WidgetId>> = RefCell::new(Tree::default());
}

create_entity! {
    pub WidgetId
}

pub struct Interactivity {}

/// main building block to create a renderable component
pub trait Widget {
    fn id(&self) -> &WidgetId;

    fn state(&mut self) -> &mut WidgetState;

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

pub trait ParentWidget: Widget + Sized {
    fn child(self, child: impl IntoView + 'static) -> Self {
        TREE.with_borrow_mut(|tree| tree.insert(*child.id(), Some(*self.id())));
        self
    }

    fn layout_rules(&mut self) -> &mut LayoutRules;

    fn padding(mut self, padding: Padding) -> Self {
        self.layout_rules().padding = padding;
        self
    }

    fn spacing(mut self, spacing: u8) -> Self {
        self.layout_rules().spacing = spacing;
        self
    }

    fn align_h(mut self, align_h: AlignH) -> Self {
        self.layout_rules().align_h = align_h;
        self
    }

    fn align_v(mut self, align_v: AlignV) -> Self {
        self.layout_rules().align_v = align_v;
        self
    }

    fn orientation(mut self, orientation: Orientation) -> Self {
        self.layout_rules().orientation = orientation;
        self
    }
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
        self.state().shape = shape;
        // let _ = shape;
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

    fn state(&mut self) -> &mut WidgetState {
        self.as_mut().state()
    }
}

impl Widget for Box<&mut dyn Widget> {
    fn id(&self) -> &WidgetId {
        self.as_ref().id()
    }

    fn state(&mut self) -> &mut WidgetState {
        self.as_mut().state()
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

pub(crate) fn window(rect: Rect) -> WindowWidget {
    WindowWidget::new(rect)
}

pub(crate) struct WindowWidget {
    id: WidgetId,
    state: WidgetState,
    layout_rules: LayoutRules,
}

impl WindowWidget {
    pub(crate) fn new(rect: Rect) -> Self {
        let layout_rules = LayoutRules::default();
        Self {
            id: ENTITY_MANAGER.with_borrow_mut(|m| m.create()),
            state: WidgetState::window(rect.width, rect.height),
            layout_rules,
        }
    }
}

impl Widget for WindowWidget {
    fn id(&self) -> &WidgetId {
        &self.id
    }
    fn state(&mut self) -> &mut WidgetState {
        &mut self.state
    }
}

impl ParentWidget for WindowWidget {
    fn layout_rules(&mut self) -> &mut LayoutRules {
        &mut self.layout_rules
    }
}

// -------------------------------------

pub fn circle() -> CircleWidget {
    CircleWidget::new()
}

pub struct CircleWidget {
    id: WidgetId,
    state: WidgetState,
}

impl CircleWidget {
    pub fn new() -> Self {
        let id = ENTITY_MANAGER.with_borrow_mut(|m| m.create());
        let state = WidgetState::default()
            .with_size(100, 100)
            .with_shape(Shape::Circle)
            .with_border_width(5.);
        Self { id, state }
    }
}

impl Widget for CircleWidget {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn state(&mut self) -> &mut WidgetState {
        &mut self.state
    }
}
