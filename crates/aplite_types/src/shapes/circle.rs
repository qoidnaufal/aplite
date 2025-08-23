use crate::Vec2f;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Circle {
    center_x: f32,
    center_y: f32,
    radius: f32,
}

impl Circle {
    pub const fn new(center: Vec2f, radius: f32) -> Self {
        Self { center_x: center.x, center_y: center.y, radius }
    }

    pub const fn center(&self) -> Vec2f {
        Vec2f::new(self.center_x, self.center_y)
    }

    pub const fn top_left(&self) -> Vec2f {
        Vec2f::new(self.center_x - self.radius, self.center_y - self.radius)
    }

    pub const fn radius(&self) -> f32 {
        self.radius
    }

    pub const fn set_center(&mut self, center: Vec2f) {
        self.center_x = center.x;
        self.center_y = center.y;
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
        self.center_x.eq(&other.center_x)
            && self.center_y.eq(&other.center_y)
            && self.radius.eq(&other.radius)
    }
}
