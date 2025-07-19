use crate::Vec2f;

#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy)]
pub struct Circle {
    center: Vec2f,
    radius: f32,
}

impl Circle {
    pub const fn new(center: Vec2f, radius: f32) -> Self {
        Self { center, radius }
    }

    pub const fn center(&self) -> Vec2f {
        self.center
    }

    pub const fn top_left(&self) -> Vec2f {
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

impl PartialOrd for Circle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.radius.partial_cmp(&other.radius)
    }
}

impl PartialEq for Circle {
    fn eq(&self, other: &Self) -> bool {
        self.center.eq(&other.center) && self.radius.eq(&other.radius)
    }
}
