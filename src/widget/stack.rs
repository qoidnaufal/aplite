use std::marker::PhantomData;
use aplite_renderer::Scene;
use aplite_storage::ComponentTuple;
use aplite_types::{Rgba, Size};

use crate::layout::{LayoutRules, Orientation};
use crate::context::Context;
use crate::view::IntoView;
use crate::widget::{Mountable, Widget};

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

struct Stack<IV: IntoView, SO> {
    widget: IV,
    marker: PhantomData<SO>
}

impl<IV: IntoView, SO> Stack<IV, SO> {
    fn new(widget: IV) -> Self {
        Self {
            widget,
            marker: PhantomData,
        }
    }
}

impl<IV: IntoView, SO: StackOrientation> Widget for Stack<IV, SO> {
    fn layout(&self, cx: &mut Context) {
        match SO::ORIENTATION {
            Orientation::Horizontal => self.widget.layout(cx),
            Orientation::Vertical => self.widget.layout(cx),
        }
    }

    fn draw(&self, scene: &mut Scene) {
        self.widget.draw(scene);
    }
}

impl<IV: IntoView, SO: StackOrientation> Mountable for Stack<IV, SO> {
    fn build(self, cx: &mut Context) {
        self.widget.build(cx);
    }
}
