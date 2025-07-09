use super::{Vector2, Vector4, Size, GpuPrimitive, NumDebugger, Fraction};

#[derive(Clone, Copy)]
pub struct Rect<T: GpuPrimitive> {
    inner: Vector4<T>,
}

impl<T: NumDebugger> std::fmt::Debug for Rect<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.inner.debug_formatter("Rect");
        write!(f, "{s}")
    }
}

impl<T: GpuPrimitive> PartialEq for Rect<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: GpuPrimitive> Eq for Rect<T> {}

impl<T: GpuPrimitive> Rect<T> {
    pub const fn new(pos: (T, T), size: (T, T)) -> Self {
        Self { inner: Vector4::new(pos.0, pos.1, size.0, size.1) }
    }

    #[inline(always)]
    pub const fn pos(&self) -> Vector2<T> {
        Vector2::new(self.inner.x(), self.inner.y())
    }

    #[inline(always)]
    pub fn set_pos(&mut self, pos: Vector2<T>) {
        self.inner[0] = pos.x();
        self.inner[1] = pos.y();
    }

    #[inline(always)]
    pub const fn x(&self) -> T { self.inner.x() }

    #[inline(always)]
    pub fn set_x(&mut self, x: T) { self.inner.set_x(x) }

    #[inline(always)]
    pub fn add_x(&mut self, x: T) { self.inner.add_x(x) }

    #[inline(always)]
    pub const fn y(&self) -> T { self.inner.y() }

    #[inline(always)]
    pub fn set_y(&mut self, y: T) { self.inner.set_y(y) }

    #[inline(always)]
    pub fn add_y(&mut self, y: T) { self.inner.add_y(y) }

    #[inline(always)]
    pub const fn size(&self) -> Size<T> {
        Size::new(self.inner.z(), self.inner.w())
    }

    #[inline(always)]
    pub fn set_size(&mut self, size: Size<T>) {
        self.inner[2] = size.width();
        self.inner[3] = size.height();
    }

    #[inline(always)]
    pub const fn width(&self) -> T { self.inner.z() }

    #[inline(always)]
    pub fn set_width(&mut self, width: T) { self.inner.set_z(width) }

    #[inline(always)]
    pub fn add_width(&mut self, width: T) { self.inner.add_z(width) }

    #[inline(always)]
    pub const fn height(&self) -> T { self.inner.w() }

    #[inline(always)]
    pub fn set_height(&mut self, height: T) { self.inner.set_w(height) }

    #[inline(always)]
    pub fn add_height(&mut self, height: T) { self.inner.add_w(height) }

    #[inline(always)]
    pub const fn l(&self) -> T { self.inner.x() }

    #[inline(always)]
    pub fn r(&self) -> T { self.x() + self.width() }

    #[inline(always)]
    pub const fn t(&self) -> T { self.inner.y() }

    #[inline(always)]
    pub fn b(&self) -> T { self.y() + self.height() }
}

impl Rect<u32> {
    pub fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
        self.set_width(self.height() * aspect_ratio)
    }

    pub fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
        self.set_height(self.width() / aspect_ratio)
    }

    pub fn f32(self) -> Rect<f32> {
        Rect { inner: self.inner.f32() }
    }
}

impl Rect<f32> {
    pub fn u32(self) -> Rect<u32> {
        Rect { inner: self.inner.u32() }
    }
}

impl From<Rect<f32>> for Rect<u32> {
    fn from(value: Rect<f32>) -> Self {
        Self { inner: value.inner.u32() }
    }
}

impl From<Rect<u32>> for Rect<f32> {
    fn from(value: Rect<u32>) -> Self {
        Self { inner: value.inner.f32() }
    }
}
