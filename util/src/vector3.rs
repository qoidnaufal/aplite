use crate::Vector2;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T: Default> Default for Vector3<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default> Vector3<T> {
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
impl<T> std::ops::Mul<T> for Vector3<T>
where T:
    std::ops::Mul<T, Output = T>
    + Copy
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// dot product
impl<T> std::ops::Mul<Self> for Vector3<T>
where T:
    std::ops::Add<T, Output = T>
    + std::ops::Mul<T, Output = T>
    + Copy
{
    type Output = T;
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl<T> std::ops::Add<Self> for Vector3<T>
where T:
    std::ops::Add<T, Output = T>
    + Copy
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> std::ops::AddAssign<Self> for Vector3<T>
where T:
    std::ops::Add<T, Output = T>
    + std::ops::AddAssign
    + Copy
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> std::ops::Sub<Self> for Vector3<T>
where T:
    std::ops::Sub<T, Output = T>
    + Copy
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> std::ops::SubAssign<Self> for Vector3<T>
where T:
    std::ops::Sub<T, Output = T>
    + std::ops::SubAssign
    + Copy
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<T> PartialEq for Vector3<T>
where T:
    PartialEq<T>
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.z == other.z
    }
}

impl<T: PartialEq<T> + Eq> Eq for Vector3<T> {}
