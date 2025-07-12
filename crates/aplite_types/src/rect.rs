use crate::num_traits::{GpuPrimitive, NumDebugger};
use crate::fraction::Fraction;
use crate::size::Size;
use crate::vector::{Vector, Vec2f, Vec2u};

#[derive(Clone, Copy)]
pub struct Rect<T: GpuPrimitive> {
    inner: Vector<4, T>,
}

impl<T: GpuPrimitive> Rect<T> {
    pub const fn new(x: T, y: T, width: T, height: T) -> Self {
        Self { inner: Vector::new_from_array([x, y, width, height]) }
    }
}

impl Rect<u32> {
    #[inline(always)]
    pub const fn pos(&self) -> Vec2u {
        Vec2u::new(self.inner.x(), self.inner.y())
    }

    #[inline(always)]
    pub const fn set_pos(&mut self, pos: Vec2u) {
        self.set_x(pos.x());
        self.set_y(pos.y());
    }

    #[inline(always)]
    pub const fn size(&self) -> Size<u32> {
        Size::new(self.inner.z(), self.inner.w())
    }

    #[inline(always)]
    pub const fn set_size(&mut self, size: Size<u32>) {
        self.set_width(size.width());
        self.set_height(size.height());
    }

    #[inline(always)]
    pub const fn x(&self) -> u32 { self.inner.x() }

    #[inline(always)]
    pub const fn set_x(&mut self, x: u32) { self.inner.set_x(x) }

    #[inline(always)]
    pub const fn add_x(&mut self, x: u32) { self.inner.add_x(x) }

    #[inline(always)]
    pub const fn y(&self) -> u32 { self.inner.y() }

    #[inline(always)]
    pub const fn set_y(&mut self, y: u32) { self.inner.set_y(y) }

    #[inline(always)]
    pub const fn add_y(&mut self, y: u32) { self.inner.add_y(y) }

    #[inline(always)]
    pub const fn width(&self) -> u32 { self.inner.z() }

    #[inline(always)]
    pub const fn set_width(&mut self, width: u32) { self.inner.set_z(width) }

    #[inline(always)]
    pub const fn add_width(&mut self, width: u32) { self.inner.add_z(width) }

    #[inline(always)]
    pub const fn height(&self) -> u32 { self.inner.w() }

    #[inline(always)]
    pub const fn set_height(&mut self, height: u32) { self.inner.set_w(height) }

    #[inline(always)]
    pub const fn add_height(&mut self, height: u32) { self.inner.add_w(height) }

    #[inline(always)]
    pub const fn l(&self) -> u32 { self.inner.x() }

    #[inline(always)]
    pub const fn r(&self) -> u32 { self.x() + self.width() }

    #[inline(always)]
    pub const fn t(&self) -> u32 { self.inner.y() }

    #[inline(always)]
    pub const fn b(&self) -> u32 { self.y() + self.height() }
}
impl Rect<f32> {
    #[inline(always)]
    pub const fn pos(&self) -> Vec2f {
        Vec2f::new(self.inner.x(), self.inner.y())
    }

    #[inline(always)]
    pub const fn set_pos(&mut self, pos: Vec2f) {
        self.set_x(pos.x());
        self.set_y(pos.y());
    }

    #[inline(always)]
    pub const fn size(&self) -> Size<f32> {
        Size::new(self.inner.z(), self.inner.w())
    }

    #[inline(always)]
    pub const fn set_size(&mut self, size: Size<f32>) {
        self.set_width(size.width());
        self.set_height(size.height());
    }

    #[inline(always)]
    pub const fn x(&self) -> f32 { self.inner.x() }

    #[inline(always)]
    pub const fn set_x(&mut self, x: f32) { self.inner.set_x(x) }

    #[inline(always)]
    pub const fn add_x(&mut self, x: f32) { self.inner.add_x(x) }

    #[inline(always)]
    pub const fn y(&self) -> f32 { self.inner.y() }

    #[inline(always)]
    pub const fn set_y(&mut self, y: f32) { self.inner.set_y(y) }

    #[inline(always)]
    pub const fn add_y(&mut self, y: f32) { self.inner.add_y(y) }

    #[inline(always)]
    pub const fn width(&self) -> f32 { self.inner.z() }

    #[inline(always)]
    pub const fn set_width(&mut self, width: f32) { self.inner.set_z(width) }

    #[inline(always)]
    pub const fn add_width(&mut self, width: f32) { self.inner.add_z(width) }

    #[inline(always)]
    pub const fn height(&self) -> f32 { self.inner.w() }

    #[inline(always)]
    pub const fn set_height(&mut self, height: f32) { self.inner.set_w(height) }

    #[inline(always)]
    pub const fn add_height(&mut self, height: f32) { self.inner.add_w(height) }

    #[inline(always)]
    pub const fn l(&self) -> f32 { self.inner.x() }

    #[inline(always)]
    pub const fn r(&self) -> f32 { self.x() + self.width() }

    #[inline(always)]
    pub const fn t(&self) -> f32 { self.inner.y() }

    #[inline(always)]
    pub const fn b(&self) -> f32 { self.y() + self.height() }
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

impl<T: GpuPrimitive> PartialEq for Rect<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: GpuPrimitive> Eq for Rect<T> {}

impl<T: NumDebugger> std::fmt::Debug for Rect<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.inner.debug_formatter("Rect");
        write!(f, "{s}")
    }
}
