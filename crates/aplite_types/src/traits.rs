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

pub trait NumDebugger: GpuPrimitive {
    fn is_signed(&self) -> bool { false }
    fn is_float(&self) -> bool { false }
}

impl NumDebugger for u8 {}
impl NumDebugger for u32 {}
impl NumDebugger for f32 {
    fn is_signed(&self) -> bool { self.is_sign_negative() }
    fn is_float(&self) -> bool { true }
}
