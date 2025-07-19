use crate::Size;

#[derive(Debug, Clone, Copy)]
pub struct Fraction {
    numerator: f32,
    denominator: f32,
}

impl Default for Fraction {
    fn default() -> Self {
        Self {
            numerator: 0.0,
            denominator: 0.0,
        }
    }
}

impl Fraction {
    #[inline(always)]
    pub const fn new(numerator: f32, denominator: f32) -> Self {
        Self { numerator, denominator }
    }
}

impl From<(f32, f32)> for Fraction {
    fn from(value: (f32, f32)) -> Self {
        Self {
            numerator: value.0,
            denominator: value.1,
        }
    }
}

impl From<(u32, u32)> for Fraction {
    fn from(value: (u32, u32)) -> Self {
        Self {
            numerator: value.0 as f32,
            denominator: value.1 as f32,
        }
    }
}

impl std::ops::Mul<Size> for Fraction {
    type Output = Size;
    fn mul(self, rhs: Size) -> Self::Output {
        Size::new(self.numerator * rhs.width, self.denominator * rhs.height)
    }
}

impl std::ops::Mul<Fraction> for u32 {
    type Output = u32;
    fn mul(self, rhs: Fraction) -> Self::Output {
        self * rhs.numerator.round() as u32 / rhs.denominator.round() as u32
    }
}

impl std::ops::Mul<Fraction> for f32 {
    type Output = f32;
    fn mul(self, rhs: Fraction) -> Self::Output {
        self * rhs.numerator / rhs.denominator
    }
}

impl std::ops::Div<Fraction> for u32 {
    type Output = u32;
    fn div(self, rhs: Fraction) -> Self::Output {
        self * rhs.denominator.round() as u32 / rhs.numerator.round() as u32
    }
}

impl std::ops::Div<Fraction> for f32 {
    type Output = f32;
    fn div(self, rhs: Fraction) -> Self::Output {
        self * rhs.denominator / rhs.numerator
    }
}
