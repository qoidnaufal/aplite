use crate::Size;

#[derive(Debug, Clone, Copy)]
pub struct Fraction<T> {
    pub numerator: T,
    pub denominator: T,
}

impl<T: Default> Default for Fraction<T> {
    fn default() -> Self {
        Self {
            numerator: T::default(),
            denominator: T::default(),
        }
    }
}

impl<T> Fraction<T> {
    pub const fn new(numerator: T, denominator: T) -> Self {
        Self { numerator, denominator }
    }
}

impl<T> std::ops::Mul<Size<T>> for Fraction<T>
where
    T: std::ops::Mul<T, Output = T>,
{
    type Output = Size<T>;
    fn mul(self, rhs: Size<T>) -> Self::Output {
        Size::new(self.numerator * rhs.width, self.denominator * rhs.height)
    }
}

impl std::ops::Mul<Fraction<u32>> for u32
{
    type Output = u32;
    fn mul(self, rhs: Fraction<u32>) -> Self::Output {
        self * rhs.numerator / rhs.denominator
    }
}

impl std::ops::Div<Fraction<u32>> for u32
{
    type Output = u32;
    fn div(self, rhs: Fraction<u32>) -> Self::Output {
        self * rhs.denominator / rhs.numerator
    }
}
