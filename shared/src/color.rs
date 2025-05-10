use super::{Vector4, GpuPrimitive, NumDebugger};

#[derive(Clone, Copy)]
pub struct Rgba<T: GpuPrimitive> {
    inner: Vector4<T>
}

impl<T: GpuPrimitive + NumDebugger> std::fmt::Debug for Rgba<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.inner.debug_formatter("Rgba");
        write!(f, "{s}")
    }
}

impl<T: GpuPrimitive> Rgba<T> {
    pub const fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { inner: Vector4::new(r, g, b, a) }
    }

    pub const fn to_slice(self) -> [T; 4] { self.inner.slice() }

    pub const fn r(&self) -> T { self.inner.x() }
    pub const fn g(&self) -> T { self.inner.y() }
    pub const fn b(&self) -> T { self.inner.z() }
    pub const fn a(&self) -> T { self.inner.w() }
}

impl Rgba<u8> {
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

impl From<Rgba<u8>> for wgpu::Color {
    fn from(val: Rgba<u8>) -> Self {
        Self {
            r: val.r() as f64 / u8::MAX as f64,
            g: val.g() as f64 / u8::MAX as f64,
            b: val.b() as f64 / u8::MAX as f64,
            a: val.a() as f64 / u8::MAX as f64,
        }
    }
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
            (val.r() * u8::MAX as f32) as u8,
            (val.g() * u8::MAX as f32) as u8,
            (val.b() * u8::MAX as f32) as u8,
            (val.a() * u8::MAX as f32) as u8,
        )
    }
}

impl From<Vector4<f32>> for Rgba<u8> {
    fn from(val: Vector4<f32>) -> Self {
        Self::new(
            (val.x() * u8::MAX as f32) as u8,
            (val.y() * u8::MAX as f32) as u8,
            (val.z() * u8::MAX as f32) as u8,
            (val.w() * u8::MAX as f32) as u8,
        )
    }
}

impl From<Rgba<u8>> for Vector4<f32> {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new(
            rgba.r() as f32 / u8::MAX as f32,
            rgba.g() as f32 / u8::MAX as f32,
            rgba.b() as f32 / u8::MAX as f32,
            rgba.a() as f32 / u8::MAX as f32,
        )
    }
}

impl From<Rgba<f32>> for Vector4<f32> {
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
