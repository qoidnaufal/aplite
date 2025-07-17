use crate::num_traits::GpuPrimitive;
use crate::vector::{Vector, Vec2u, Vec2f};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Circle<T: GpuPrimitive> {
    center: Vector<2, T>,
    radius: T,
}

impl Circle<u32> {
    pub const fn new(center: Vec2u, radius: u32) -> Self {
        Self {
            center,
            radius
        }
    }

    pub const fn center(&self) -> Vec2u {
        self.center
    }

    pub const fn radius(&self) -> u32 {
        self.radius
    }

    pub const fn set_center(&mut self, center: Vec2u) {
        self.center = center;
    }

    pub const fn set_radius(&mut self, r: u32) {
        self.radius = r;
    }
}

impl Circle<f32> {
    pub const fn new(center: Vec2f, radius: f32) -> Self {
        Self {
            center,
            radius
        }
    }

    pub const fn center(&self) -> Vec2f {
        self.center
    }

    pub const fn radius(&self) -> f32 {
        self.radius
    }

    pub const fn set_center(&mut self, center: Vec2f) {
        self.center = center;
    }

    pub const fn set_radius(&mut self, r: f32) {
        self.radius = r;
    }
}

impl<T: GpuPrimitive> PartialOrd for Circle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.radius.partial_cmp(&other.radius)
    }
}

impl<T: GpuPrimitive> PartialEq for Circle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.center.eq(&other.center) && self.radius.eq(&other.radius)
    }
}

impl<T: GpuPrimitive> std::fmt::Debug for Circle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Circle")
            .field("center", &self.center.inner)
            .field("radius", &self.radius)
            .finish()
    }
}
