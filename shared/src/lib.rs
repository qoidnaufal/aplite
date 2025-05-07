mod color;
mod vector;
mod size;
mod matrix;
mod fraction;

pub use size::Size;
pub use matrix::Matrix4x4;
pub use fraction::Fraction;
pub use color::Rgba;
pub use vector::{
    Vector2,
    Vector3,
    Vector4,
};

pub trait GpuPrimitive where Self:
    Copy + Clone + PartialEq + PartialOrd + Default
    + std::ops::Add<Self, Output = Self> + std::ops::AddAssign<Self>
    + std::ops::Div<Self, Output = Self> + std::ops::DivAssign<Self>
    + std::ops::Mul<Self, Output = Self> + std::ops::MulAssign<Self>
    + std::ops::Rem<Self, Output = Self> + std::ops::RemAssign<Self>
    + std::ops::Sub<Self, Output = Self> + std::ops::SubAssign<Self>
    + std::fmt::Debug + std::fmt::Display {}

impl GpuPrimitive for u8 {}
impl GpuPrimitive for u32 {}
impl GpuPrimitive for f32 {}

pub fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

pub(crate) fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        return a;
    }
    gcd(b, a % b)
}
