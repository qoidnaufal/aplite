use crate::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Default> Vector2<T> {
    pub fn new() -> Self {
        Self { x: T::default(), y: T::default() }
    }
}

impl std::ops::Mul<f32> for Vector2<f32> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Mul<u32> for Vector2<u32> {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Add<Self> for Vector2<f32> {
    type Output = Vector2<f32>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl std::ops::AddAssign<Self> for Vector2<f32> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl std::ops::Sub<Self> for Vector2<f32> {
    type Output = Vector2<f32>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl std::ops::SubAssign<Self> for Vector2<f32> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<Vector2<u32>> for Vector2<f32> {
    fn from(val: Vector2<u32>) -> Self {
        Self {
            x: val.x as _,
            y: val.y as _,
        }
    }
}

impl From<Vector2<f32>> for Vector2<u32> {
    fn from(val: Vector2<f32>) -> Self {
        Self {
            x: val.x as _,
            y: val.y as _,
        }
    }
}

impl<T> From<Vector3<T>> for Vector2<T> {
    fn from(val: Vector3<T>) -> Self {
        Self {
            x: val.x,
            y: val.y
        }
    }
}

impl<T> From<(T, T)> for Vector2<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl PartialEq for Vector2<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Vector2<u32> {}

impl PartialEq for Vector2<f32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Vector2<f32> {}

