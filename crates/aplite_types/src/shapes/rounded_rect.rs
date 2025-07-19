use crate::corner_radius::CornerRadius;
use crate::size::Size;
use crate::Vec2f;

use super::Rect;

#[repr(C, align(16))]
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
    pub const fn from_rect_radius_splat(rect: Rect, radius: f32) -> Self {
        Self::from_rect_radius(rect, CornerRadius::splat(radius))
    }

    #[inline(always)]
    pub fn from_points_radius(p1: Vec2f, p2: Vec2f, radius: CornerRadius) -> Self {
        let rect = Rect::from_points(p1, p2);
        Self::from_rect_radius(rect, radius)
    }

    #[inline(always)]
    pub const fn from_point_size_radius(point: Vec2f, size: Size, radius: CornerRadius) -> Self {
        let rect = Rect::from_point_size(point, size);
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
    pub const fn set_radius_all(&mut self, val: f32) {
        self.radius.set_all(val)
    }

    /// set each corner radius in counter clockwise direction starting from top left
    #[inline(always)]
    pub const fn set_radius_each(&mut self, tl: f32, bl: f32, br: f32, tr: f32) {
        self.radius.set_each(tl, bl, br, tr)
    }
}
