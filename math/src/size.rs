use std::ops::Deref;

#[derive(Debug,Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T>
where T:
    Default
    + std::ops::Add<T, Output = T>
    + std::ops::AddAssign
    + std::ops::Sub<T, Output = T>
    + std::ops::SubAssign
    + std::ops::Mul<T, Output = T>
    + std::ops::MulAssign
    + std::ops::Div<T, Output = T>
    + std::ops::DivAssign
    + Copy
{
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl From<Size<u32>> for Size<f32> {
    fn from(value: Size<u32>) -> Self {
        Self  {
            width: value.width as _,
            height: value.height as _,
        }
    }
}

impl<T> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl<T> Deref for Size<T>
where T:
    Deref<Target = T>
    + Copy
{
    type Target = Self;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<T> PartialEq for Size<T>
where T:
    PartialEq<T>
{
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl<T: PartialEq + Eq> Eq for Size<T> {}

impl<T> std::ops::Div<T> for Size<T>
where T:
    std::ops::Div<T, Output = T>
    + Copy
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

impl<T> std::ops::Div for Size<T>
where T:
    std::ops::Div<T, Output = T>
    + Copy
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

impl<T> std::ops::Add for Size<T>
where T:
    std::ops::Add<T, Output = T>
    + std::ops::AddAssign
    + Copy
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T> std::ops::AddAssign for Size<T>
where T:
    std::ops::Add<T, Output = T>
    + std::ops::AddAssign
    + Copy
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> std::ops::Sub for Size<T>
where T:
    std::ops::Sub<T, Output = T>
    + std::ops::SubAssign
    + Copy
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl<T> std::ops::Mul<T> for Size<T>
where T:
    std::ops::Mul<T, Output = T>
    + std::ops::MulAssign
    + Copy
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl<T> std::ops::MulAssign<T> for Size<T>
where T:
    std::ops::Mul<T, Output = T>
    + std::ops::MulAssign
    + Copy
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs
    }
}

impl<T> PartialOrd for Size<T>
where T:
    PartialOrd<T> + Ord + PartialEq<T> + Eq
    + std::ops::Mul<T, Output = T>
    + Copy
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Size<T>
where T:
    PartialOrd<T> + Ord + PartialEq<T> + Eq
    + std::ops::Mul<T, Output = T>
    + Copy
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.width * self.height;
        let b = other.width * other.height;
        (a).cmp(&b)
    }
}
