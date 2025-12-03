use aplite_renderer::Scene;

use crate::layout::{LayoutRules, Orientation};
use crate::context::Context;

use super::{Widget, ParentWidget};

pub fn v_stack() -> VStack {
    VStack::new()
}

pub fn h_stack() -> HStack {
    HStack::new()
}

pub struct VStack {
    layout_rules: LayoutRules,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            layout_rules: LayoutRules {
                orientation: Orientation::Vertical,
                ..Default::default()
            },
        }
    }
}

impl Widget for VStack {
    fn build(self, cx: &mut Context) -> aplite_storage::Entity {
        todo!()
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl ParentWidget for VStack {}

pub struct HStack {
    layout_rules: LayoutRules,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            layout_rules: LayoutRules {
                orientation: Orientation::Horizontal,
                ..Default::default()
            },
        }
    }
}

impl Widget for HStack {
    fn build(self, cx: &mut Context) -> aplite_storage::Entity {
        todo!()
    }

    fn layout(&mut self, cx: &mut Context) {
        todo!()
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }
}

impl ParentWidget for HStack {}

impl std::fmt::Debug for VStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VStack")
    }
}

impl std::fmt::Debug for HStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HStack")
    }
}
