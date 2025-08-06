use crate::vector::{Vec2f, Vec2u};

/// The difference between [`Point`] and [`Vec2f`] is, [`Point`] has less functionality than [`Vec2f`]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

impl Point {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn splat(val: f32) -> Self {
        Self::new(val, val)
    }

    #[inline(always)]
    pub const fn from_array(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }

    #[inline(always)]
    pub fn into_array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    #[inline(always)]
    pub fn vec2f(self) -> Vec2f {
        Vec2f::new(self.x, self.y)
    }

    #[inline(always)]
    pub const fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y)
        )
    }

    #[inline(always)]
    pub const fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y)
        )
    }

    #[inline(always)]
    pub const fn clamp(self, start: Self, end: Self) -> Self {
        self.max(start).min(end)
    }
}

impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<[f32; 2]> for Point {
    fn from(arr: [f32; 2]) -> Self {
        Self::from_array(arr)
    }
}

impl From<Vec2f> for Point {
    fn from(value: Vec2f) -> Self {
        value.point()
    }
}

impl From<Vec2u> for Point {
    fn from(value: Vec2u) -> Self {
        value.point()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
