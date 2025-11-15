use aplite_types::{Rgba, Unit};
use aplite_types::theme::basic;

use crate::layout::{LayoutRules, Orientation};
use crate::view::IntoView;

use super::{Widget, ParentWidget};

pub fn v_stack() -> VStack {
    VStack::new()
}

pub fn h_stack() -> HStack {
    HStack::new()
}

pub struct VStack {
    width: Unit,
    height: Unit,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    background: Rgba,
    border_color: Rgba,
    border_width: f32,
    layout_rules: LayoutRules,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            width: Unit::Fit,
            height: Unit::Fit,
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            background: basic::TRANSPARENT,
            border_color: basic::TRANSPARENT,
            border_width: 0.,
            layout_rules: LayoutRules {
                orientation: Orientation::Vertical,
                ..Default::default()
            },
        }
    }
}

impl IntoView for VStack {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl Widget for VStack {}

impl ParentWidget for VStack {}

pub struct HStack {
    width: Unit,
    height: Unit,
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
    background: Rgba,
    border: Rgba,
    border_width: f32,
    layout_rules: LayoutRules,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            width: Unit::Fit,
            height: Unit::Fit,
            min_width: Some(1.),
            min_height: Some(1.),
            max_width: None,
            max_height: None,
            background: basic::TRANSPARENT,
            border: basic::TRANSPARENT,
            border_width: 0.,
            layout_rules: LayoutRules {
                orientation: Orientation::Horizontal,
                ..Default::default()
            },
        }
    }
}

impl IntoView for HStack {
    type View = Self;

    fn into_view(self) -> Self::View {
        self
    }
}

impl Widget for HStack {}

impl ParentWidget for HStack {}
