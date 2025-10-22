use aplite_types::{Rgba, Unit};
use aplite_renderer::{Shape, Scene};

use crate::layout::{Layout, LayoutRules, Orientation};
use crate::view::IntoView;

use super::{WidgetId, Widget};

pub fn v_stack<F>() -> VStack {
    VStack::new()
}

pub fn h_stack<F>() -> HStack {
    HStack::new()
}

pub struct VStack {
    id: WidgetId,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    background: Rgba,
    border: Rgba,
    border_width: f32,
    layout_rules: LayoutRules,
    children: Vec<Box<dyn IntoView>>,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new_id(),
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            background: Rgba::TRANSPARENT,
            border: Rgba::TRANSPARENT,
            border_width: 0.,
            layout_rules: LayoutRules {
                orientation: Orientation::Vertical,
                ..Default::default()
            },
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoView + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl Widget for VStack {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn draw(&self, scene: &mut Scene) {
        todo!()
    }

    fn layout(&self, cx: &mut Layout) {
        todo!()
    }
}

pub struct HStack {
    id: WidgetId,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    background: Rgba,
    border: Rgba,
    border_width: f32,
    layout_rules: LayoutRules,
    children: Vec<Box<dyn IntoView>>,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new_id(),
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            background: Rgba::TRANSPARENT,
            border: Rgba::TRANSPARENT,
            border_width: 0.,
            layout_rules: LayoutRules {
                orientation: Orientation::Horizontal,
                ..Default::default()
            },
            children: Vec::new(),
        }
    }
}

impl Widget for HStack {
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
