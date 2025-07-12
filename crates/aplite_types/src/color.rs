use crate::num_traits::{GpuPrimitive, NumDebugger};
use crate::vector::{Vector, Vec4f};

pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
    Rgba::new(r, g, b, a)
}

/// value must be between 0.0 and 1.0
pub const fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Rgba<f32> {
    Rgba::new(r, g, b, a)
}

#[derive(Clone, Copy)]
pub struct Rgba<T: GpuPrimitive> {
    inner: Vector<4, T>
}

impl<T: GpuPrimitive + NumDebugger> std::fmt::Debug for Rgba<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.inner.debug_formatter("Rgba");
        write!(f, "{s}")
    }
}

impl<T: GpuPrimitive> Rgba<T> {
    pub const fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { inner: Vector::new_from_array([r, g, b, a]) }
    }

    #[inline(always)]
    pub const fn as_slice(&self) -> &[T] { self.inner.as_slice() }

    #[inline(always)]
    pub const fn r(&self) -> T { self.inner.inner[0] }

    #[inline(always)]
    pub const fn g(&self) -> T { self.inner.inner[1] }

    #[inline(always)]
    pub const fn b(&self) -> T { self.inner.inner[2] }

    #[inline(always)]
    pub const fn a(&self) -> T { self.inner.inner[3] }
}

impl Rgba<u8> {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);
    pub const PURPLE: Self = Self::new(111, 72, 234, 255);
    pub const LIGHT_GRAY: Self = Rgba::new(69, 69, 69, 255);
    pub const DARK_GRAY: Self = Self::new(30, 30, 30, 255);
    pub const DARK_GREEN: Self = Self::new(10, 30, 15, 255);
}

// taken straight up from kludgine

impl From<Rgba<u8>> for u32 {
    fn from(rgba: Rgba<u8>) -> Self {
        ((rgba.r() as u32) << 24)
        | ((rgba.g() as u32) << 16)
        | ((rgba.b() as u32) << 8)
        | (rgba.a() as u32)
    }
}

impl From<u32> for Rgba<u8> {
    fn from(num: u32) -> Self {
        let r = (num >> 24) as u8;
        let g = ((num >> 16) & 0xFF) as u8;
        let b = ((num >> 8) & 0xFF) as u8;
        let a = (num & 0xFF) as u8;
        Self::new(r, g, b, a)
    }
}

// type conversion

impl Rgba<f32> {
    pub fn u8(self) -> Rgba<u8> {
        self.into()
    }

    pub fn to_vec4f(self) -> Vec4f {
        self.into()
    }
}

impl Rgba<u8> {
    pub fn f32(self) -> Rgba<f32> {
        self.into()
    }

    pub fn to_vec4f(self) -> Vec4f {
        self.f32().into()
    }
}

impl From<Rgba<u8>> for Rgba<f32> {
    fn from(val: Rgba<u8>) -> Self {
        Self::new(
            val.r() as f32 / u8::MAX as f32,
            val.g() as f32 / u8::MAX as f32,
            val.b() as f32 / u8::MAX as f32,
            val.a() as f32 / u8::MAX as f32,
        )
    }
}

impl From<Rgba<f32>> for Rgba<u8> {
    fn from(val: Rgba<f32>) -> Self {
        Self::new(
            (val.r() * u8::MAX as f32).round() as u8,
            (val.g() * u8::MAX as f32).round() as u8,
            (val.b() * u8::MAX as f32).round() as u8,
            (val.a() * u8::MAX as f32).round() as u8,
        )
    }
}

impl From<Vec4f> for Rgba<u8> {
    fn from(val: Vec4f) -> Self {
        Self::new(
            (val.x() * u8::MAX as f32).round() as u8,
            (val.y() * u8::MAX as f32).round() as u8,
            (val.z() * u8::MAX as f32).round() as u8,
            (val.w() * u8::MAX as f32).round() as u8,
        )
    }
}

impl From<Rgba<u8>> for Vec4f {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new(
            rgba.r() as f32 / u8::MAX as f32,
            rgba.g() as f32 / u8::MAX as f32,
            rgba.b() as f32 / u8::MAX as f32,
            rgba.a() as f32 / u8::MAX as f32,
        )
    }
}

impl From<Rgba<f32>> for Vec4f {
    fn from(rgba: Rgba<f32>) -> Self {
        Self::new(
            rgba.r(),
            rgba.g(),
            rgba.b(),
            rgba.a(),
        )
    }
}

impl<T: GpuPrimitive> PartialEq for Rgba<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Rgba<u8> {}
