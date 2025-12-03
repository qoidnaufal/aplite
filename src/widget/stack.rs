use aplite_renderer::Scene;
use aplite_storage::Entity;

use crate::layout::{LayoutRules, Orientation};
use crate::context::Context;
use crate::view::{ViewStorage, IntoView, View};

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

impl ParentWidget for VStack {}

// impl IntoView for VStack {
//     fn into_view<'a>(self) -> View<'a> {
//         View::new(self)
//     }
// }

/*
#########################################################
#                                                       #
#                         HStack                        #
#                                                       #
#########################################################
*/

#[derive(Debug)]
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

// impl IntoView for HStack {
//     fn into_view<'a>(self) -> View<'a> {
//         View::new(self)
//     }
// }

impl ParentWidget for HStack {}
