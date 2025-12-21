use std::marker::PhantomData;

use aplite_renderer::Scene;
use aplite_storage::Entity;

use crate::layout::{LayoutRules, Orientation};
use crate::context::Context;
use crate::view::{IntoView, View};
use crate::widget::Widget;

pub fn h_stack<IV: IntoView>(widget: IV) -> impl IntoView {
    Stack::<IV, Horizontal>::new(widget)
}

pub fn v_stack<IV: IntoView>(widget: IV) -> impl IntoView {
    Stack::<IV, Vertical>::new(widget)
}

trait StackOrientation {
    const ORIENTATION: Orientation;
}

struct Horizontal; impl StackOrientation for Horizontal {
    const ORIENTATION: Orientation = Orientation::Horizontal;
}

struct Vertical; impl StackOrientation for Vertical {
    const ORIENTATION: Orientation = Orientation::Vertical;
}

struct Stack<IV, D> {
    widget: IV,
    marker: PhantomData<D>
}

impl<IV, D> Stack<IV, D> {
    fn new(widget: IV) -> Self {
        Self {
            widget,
            marker: PhantomData,
        }
    }
}

impl<IV: IntoView, D: StackOrientation> Widget for Stack<IV, D> {
    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}
