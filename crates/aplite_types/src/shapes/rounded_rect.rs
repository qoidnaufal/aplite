use crate::num_traits::GpuPrimitive;
use crate::vector::{Vector, Vec2u, Vec2f};
use crate::corner_radius::CornerRadius;
use crate::size::Size;

use super::Rect;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RoundedRect<T: GpuPrimitive> {
    inner: Vector<4, T>,
    radius: CornerRadius<T>,
}

impl<T: GpuPrimitive> RoundedRect<T> {
    pub const fn new(rect: Rect<T>, radius: CornerRadius<T>) -> Self {
        Self {
            inner: Vector::new_from_array(rect.into_array()),
            radius,
        }
    }

    pub const fn size(&self) -> Size<T> {
        Size::new(self.inner.inner[2], self.inner.inner[3])
    }

    pub const fn radius(&self) -> CornerRadius<T> {
        self.radius
    }

    pub const fn set_pos(&mut self, x: T, y: T) {
        self.inner.inner[0] = x;
        self.inner.inner[1] = y;
    }

    pub const fn set_size(&mut self, width: T, height: T) {
        self.inner.inner[2] = width;
        self.inner.inner[3] = height;
    }

    pub const fn set_radius_all(&mut self, val: T) {
        self.radius.set_all(val);
    }

    /// set each corner radius in counter clockwise direction starting from top left
    pub const fn set_radius_each(&mut self, tl: T, bl: T, br: T, tr: T) {
        self.radius.set_each(tl, bl, br, tr);
    }
}

impl RoundedRect<u32> {
    pub const fn from_pos_size_radius(pos: Vec2u, size: Size<u32>, radius: CornerRadius<u32>) -> Self {
        Self {
            inner: Vector::new_from_array([
                pos.x(),
                pos.y(),
                size.width(),
                size.height(),
            ]),
            radius,
        }
    }

    pub fn from_points_radius(p1: Vec2u, p2: Vec2u, radius: CornerRadius<u32>) -> Self {
        Self {
            inner: Vector::new_from_array(Rect::<u32>::from_points(p1, p2).into_array()),
            radius,
        }
    }

    pub const fn pos(&self) -> Vec2u {
        Vec2u::new(self.inner.x(), self.inner.y())
    }
}

impl RoundedRect<f32> {
    pub const fn from_pos_size_radius(pos: Vec2f, size: Size<f32>, radius: CornerRadius<f32>) -> Self {
        Self {
            inner: Vector::new_from_array([
                pos.x(),
                pos.y(),
                size.width(),
                size.height(),
            ]),
            radius,
        }
    }

    pub const fn from_points_radius(p1: Vec2f, p2: Vec2f, radius: CornerRadius<f32>) -> Self {
        Self {
            inner: Vector::new_from_array(Rect::<f32>::from_points(p1, p2).into_array()),
            radius,
        }
    }

    pub const fn pos(&self) -> Vec2f {
        Vec2f::new(self.inner.x(), self.inner.y())
    }
}

impl<T: GpuPrimitive> std::fmt::Debug for RoundedRect<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoundedRect")
            .field("pos", &[self.inner.inner[0], self.inner.inner[1]])
            .field("size", &[self.inner.inner[2], self.inner.inner[3]])
            .field("radius", &self.radius)
            .finish()
    }
}
