use crate::Vector2;

#[derive(Debug, Clone, Copy)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T> Vector3<T>
where T:
    Default
    + std::ops::Add
    + std::ops::AddAssign
    + std::ops::Sub
    + std::ops::SubAssign
    + std::ops::Mul
    + std::ops::MulAssign
    + std::ops::Div
    + std::ops::DivAssign
{
    pub fn new() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T: Default> From<Vector2<T>> for Vector3<T> {
    fn from(val: Vector2<T>) -> Self {
        Self {
            x: val.x,
            y: val.y,
            z: T::default(),
        }
    }
}

// scalar multiplication
impl std::ops::Mul<f32> for Vector3<f32> {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// scalar multiplication
impl std::ops::Mul<u32> for Vector3<u32> {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// dot product
impl std::ops::Mul<Self> for Vector3<f32> {
    type Output = f32;
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

// dot product
impl std::ops::Mul<Self> for Vector3<u32> {
    type Output = u32;
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl std::ops::Add<Self> for Vector3<f32> {
    type Output = Vector3<f32>;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::AddAssign<Self> for Vector3<f32> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl std::ops::Sub<Self> for Vector3<f32> {
    type Output = Vector3<f32>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::SubAssign<Self> for Vector3<f32> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl PartialEq for Vector3<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.z == other.z
    }
}

impl Eq for Vector3<u32> {}

impl PartialEq for Vector3<f32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.z == other.z
    }
}

impl Eq for Vector3<f32> {}

