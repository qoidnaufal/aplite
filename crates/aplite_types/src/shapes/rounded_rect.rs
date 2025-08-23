use crate::corner_radius::CornerRadius;
use crate::size::Size;
use crate::Vec2f;

use super::Rect;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct RoundedRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: CornerRadius,
}

impl RoundedRect {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, width: f32, height: f32, radius: CornerRadius) -> Self {
        Self { x, y, width, height, radius }
    }

    #[inline(always)]
    pub const fn from_rect_radius(rect: Rect, radius: CornerRadius) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            radius,
        }
    }

    #[inline(always)]
    pub const fn from_rect_radius_splat(rect: Rect, radius: u8) -> Self {
        Self::from_rect_radius(rect, CornerRadius::splat(radius))
    }

    #[inline(always)]
    pub fn from_vec2f_radius(v1: Vec2f, v2: Vec2f, radius: CornerRadius) -> Self {
        let rect = Rect::from_vec2f(v1, v2);
        Self::from_rect_radius(rect, radius)
    }

    #[inline(always)]
    pub const fn from_point_size_radius(point: Vec2f, size: Size, radius: CornerRadius) -> Self {
        let rect = Rect::from_vec2f_size(point, size);
        Self::from_rect_radius(rect, radius)
    }

    #[inline(always)]
    pub const fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    #[inline(always)]
    pub const fn radius(&self) -> CornerRadius {
        self.radius
    }

    #[inline(always)]
    pub const fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    #[inline(always)]
    pub const fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub const fn set_radius(&mut self, radius: CornerRadius) {
        self.radius = radius
    }

    #[inline(always)]
    pub const fn set_radius_all(&mut self, val: u8) {
        self.radius.set_all(val)
    }

    /// set each corner radius in counter clockwise direction starting from top left
    #[inline(always)]
    pub const fn set_radius_each(&mut self, tl: u8, bl: u8, br: u8, tr: u8) {
        self.radius.set_each(tl, bl, br, tr)
    }
}
