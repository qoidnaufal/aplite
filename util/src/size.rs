use std::ops::Deref;
use winit::dpi::PhysicalSize;

use crate::{gcd, Fraction};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T: Default> Default for Size<T> {
    fn default() -> Self {
        Self {
            width: T::default(),
            height: T::default(),
        }
    }
}

impl<T> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl Size<u32> {
    pub fn aspect_ratio(&self) -> Fraction<u32> {
        let gcd = gcd(self.width, self.height);
        Fraction::new(self.width / gcd, self.height / gcd)
    }

    pub fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
        self.width = self.height * aspect_ratio;
    }

    pub fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
        self.height = self.width / aspect_ratio;
    }
}

impl<T> Size<T>
where
    T: PartialOrd + Ord,
{
    pub fn max(self, min_width: Option<T>, min_height: Option<T>) -> Self {
        let width = if let Some(min_width) = min_width {
            self.width.max(min_width)
        } else { self.width };
        let height = if let Some(min_height) = min_height {
            self.height.max(min_height)
        } else { self.height };
        Self {
            width,
            height,
        }
    }

    pub fn min(self, max_width: Option<T>, max_height: Option<T>) -> Self {
        let width = if let Some(max_width) = max_width {
            self.width.min(max_width)
        } else { self.width };
        let height = if let Some(max_height) = max_height {
            self.height.min(max_height)
        } else { self.height };
        Self {
            width,
            height,
        }
    }
}

impl From<Size<u32>> for PhysicalSize<u32> {
    fn from(size: Size<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl From<PhysicalSize<u32>> for Size<u32> {
    fn from(p: PhysicalSize<u32>) -> Self {
        Self::new(p.width, p.height)
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

impl From<Size<f32>> for Size<u32> {
    fn from(value: Size<f32>) -> Self {
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

impl<T> PartialEq for Size<T>
where
    T: PartialEq<T>
{
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl<T> std::ops::Mul<Fraction<T>> for Size<T>
where
    T: std::ops::Mul<T, Output = T>
{
    type Output = Self;
    fn mul(self, rhs: Fraction<T>) -> Self::Output {
        Self {
            width: self.width * rhs.numerator,
            height: self.height * rhs.denominator,
        }
    }
}

impl<T: PartialEq + Eq> Eq for Size<T> {}
