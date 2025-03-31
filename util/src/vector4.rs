use crate::{Vector2, Vector3};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> Vector4<T>
where T:
    std::ops::Mul<T, Output = T>
    + std::ops::Add<T, Output = T>,
{
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }

    pub fn dot(self, rhs: Self) -> T {
        self.x * rhs.x
        + self.y * rhs.y
        + self.z * rhs.z
        + self.w * rhs.w
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl From<Vector3<f32>> for Vector4<f32> {
    fn from(v3: Vector3<f32>) -> Self {
        Self {
            x: v3.x,
            y: v3.y,
            z: v3.z,
            w: 1.0,
        }
    }
}

impl From<Vector2<f32>> for Vector4<f32> {
    fn from(v2: Vector2<f32>) -> Self {
        Self {
            x: v2.x,
            y: v2.y,
            z: 1.0,
            w: 1.0,
        }
    }
}


