use crate::{GpuPrimitive, Size};

#[derive(Debug, Clone, Copy)]
pub struct Fraction<T: GpuPrimitive> {
    numerator: T,
    denominator: T,
}

impl<T: GpuPrimitive> Default for Fraction<T> {
    fn default() -> Self {
        Self {
            numerator: T::default(),
            denominator: T::default(),
        }
    }
}

impl<T: GpuPrimitive> Fraction<T> {
    pub const fn new(numerator: T, denominator: T) -> Self {
        Self { numerator, denominator }
    }
}

impl<T: GpuPrimitive> From<(T, T)> for Fraction<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            numerator: value.0,
            denominator: value.1,
        }
    }
}

impl<T: GpuPrimitive> std::ops::Mul<Size<T>> for Fraction<T> {
    type Output = Size<T>;
    fn mul(self, rhs: Size<T>) -> Self::Output {
        Size::new(self.numerator * rhs.width(), self.denominator * rhs.height())
    }
}

impl std::ops::Mul<Fraction<u32>> for u32 {
    type Output = u32;
    fn mul(self, rhs: Fraction<u32>) -> Self::Output {
        self * rhs.numerator / rhs.denominator
    }
}

impl std::ops::Div<Fraction<u32>> for u32 {
    type Output = u32;
    fn div(self, rhs: Fraction<u32>) -> Self::Output {
        self * rhs.denominator / rhs.numerator
    }
}
