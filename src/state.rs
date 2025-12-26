use aplite_renderer::Shape;
use aplite_types::{
    CornerRadius, Matrix3x2, Paint, Rect
};

use crate::layout::{AlignH, AlignV, Orientation, Padding};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flag(u8);

impl Flag {
    pub(crate) const VISIBLE: u8 = 1 << 0;
    pub(crate) const FOCUSED: u8 = 1 << 1;
    pub(crate) const NEEDS_LAYOUT: u8 = 1 << 2;
    pub(crate) const NEEDS_REDRAW: u8 = 1 << 3;
    const DEFAULT: u8 = Self::VISIBLE | Self::NEEDS_LAYOUT | Self::NEEDS_REDRAW;

    pub const fn new() -> Self {
        Self(Self::DEFAULT)
    }

    pub const fn is_visible(&self) -> bool {
        self.0 & Self::VISIBLE == Self::VISIBLE
    }

    pub const fn is_focused(&self) -> bool {
        self.0 & Self::FOCUSED == Self::FOCUSED
    }

    pub const fn needs_layout(&self) -> bool {
        self.0 & Self::NEEDS_LAYOUT == Self::NEEDS_LAYOUT
    }

    pub const fn needs_redraw(&self) -> bool {
        self.0 & Self::NEEDS_REDRAW == Self::NEEDS_REDRAW
    }

    pub const fn set_visible(&mut self) {
        self.0 |= self.0 ^ Self::VISIBLE
    }

    pub const fn set_focused(&mut self) {
        self.0 |= self.0 ^ Self::FOCUSED
    }

    pub const fn set_needs_layout(&mut self) {
        self.0 |= self.0 ^ Self::NEEDS_LAYOUT
    }

    pub const fn set_needs_redraw(&mut self) {
        self.0 |= self.0 ^ Self::NEEDS_REDRAW
    }

    pub const fn set_hidden(&mut self) {
        self.0 ^= Self::VISIBLE
    }

    pub const fn set_unfocused(&mut self) {
        self.0 ^= Self::FOCUSED
    }

    pub const fn set_finished_layout(&mut self) {
        self.0 ^= Self::NEEDS_LAYOUT
    }

    pub const fn set_finished_redraw(&mut self) {
        self.0 ^= Self::NEEDS_REDRAW
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

#[derive(Clone)]
pub struct Border {
    pub(crate) paint: Paint,
    pub(crate) width: f32,
}

#[derive(Clone)]
pub struct Background(Paint);

#[derive(Clone)]
pub struct Spacing(u8);

pub type DefaultRenderComponent = (Rect, Matrix3x2, Background, Border, Shape, CornerRadius);

pub type ContainerComponent = (Rect, Orientation, AlignH, AlignV, Padding, Spacing);
