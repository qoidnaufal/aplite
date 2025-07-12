use crate::fraction::Fraction;
use crate::num_traits::{GpuPrimitive, NumDebugger};
use crate::vector::Vector;

/// corresponds to [`winit::dpi::LogicalSize<T>`]
#[derive(Clone, Copy)]
pub struct Size<T: GpuPrimitive> {
    inner: Vector<2, T>
}

impl<T: GpuPrimitive + NumDebugger> std::fmt::Debug for Size<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.inner.debug_formatter("Size");
        write!(f, "{s}")
    }
}

impl<T: GpuPrimitive> Default for Size<T> {
    fn default() -> Self {
        Self {
            inner: Vector::default()
        }
    }
}

impl<T: GpuPrimitive> Size<T> {
    #[inline(always)]
    pub const fn new(width: T, height: T) -> Self {
        Self { inner: Vector::new_from_array([width, height]) }
    }

    #[inline(always)]
    pub const fn width(&self) -> T { self.inner.inner[0] }

    #[inline(always)]
    pub const fn height(&self) -> T { self.inner.inner[1] }

    #[inline(always)]
    pub const fn set_width(&mut self, val: T) { self.inner.inner[0] = val }

    #[inline(always)]
    pub const fn set_height(&mut self, val: T) { self.inner.inner[1] = val }
}

impl Size<u32> {
    #[inline(always)]
    pub const fn add_width(&mut self, val: u32) { self.inner.add_x(val) }

    #[inline(always)]
    pub const fn add_height(&mut self, val: u32) { self.inner.add_y(val) }

    #[inline(always)]
    pub const fn mul_width(&mut self, val: u32) { self.inner.mul_x(val) }

    #[inline(always)]
    pub const fn mul_height(&mut self, val: u32) { self.inner.mul_y(val) }

    #[inline(always)]
    pub const fn area(&self) -> u32 { self.width() * self.height() }

    #[inline(always)]
    pub const fn diagonal(&self) -> u32 {
        (self.width().pow(2) + self.height().pow(2)).isqrt()
    }

    pub fn aspect_ratio(&self) -> Fraction<u32> {
        let gcd = gcd(self.width(), self.height());
        Fraction::new(self.width() / gcd, self.height() / gcd)
    }

    pub fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
        self.set_width(self.height() * aspect_ratio)
    }

    pub fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
        self.set_height(self.width() / aspect_ratio)
    }

    #[inline(always)]
    pub fn f32(self) -> Size<f32> {
        Size { inner: self.inner.f32() }
    }
}

impl Size<f32> {
    #[inline(always)]
    pub const fn add_width(&mut self, val: f32) { self.inner.add_x(val) }

    #[inline(always)]
    pub const fn add_height(&mut self, val: f32) { self.inner.add_y(val) }

    #[inline(always)]
    pub const fn mul_width(&mut self, val: f32) { self.inner.mul_x(val) }

    #[inline(always)]
    pub const fn mul_height(&mut self, val: f32) { self.inner.mul_y(val) }

    #[inline(always)]
    pub const fn area(&self) -> f32 { self.width() * self.height() }

    #[inline(always)]
    pub fn diagonal(&self) -> f32 {
        (self.width().powi(2) + self.height().powi(2)).sqrt()
    }

    #[inline(always)]
    pub fn u32(self) -> Size<u32> {
        Size { inner: self.inner.u32() }
    }
}

impl<T: GpuPrimitive + PartialOrd + Ord> Size<T> {
    pub fn max(self, min_width: Option<T>, min_height: Option<T>) -> Self {
        let width = if let Some(min_width) = min_width {
            self.width().max(min_width)
        } else { self.width() };
        let height = if let Some(min_height) = min_height {
            self.height().max(min_height)
        } else { self.height() };
        Self::new(width, height)
    }

    pub fn min(self, max_width: Option<T>, max_height: Option<T>) -> Self {
        let width = if let Some(max_width) = max_width {
            self.width().min(max_width)
        } else { self.width() };
        let height = if let Some(max_height) = max_height {
            self.height().min(max_height)
        } else { self.height() };
        Self::new(width, height)
    }
}

// arithmetic operation

impl<T: GpuPrimitive> std::ops::Mul<T> for Size<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.width() * rhs, self.height() * rhs)
    }
}

impl<T: GpuPrimitive> std::ops::MulAssign<T> for Size<T> {
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs
    }
}

impl<T: GpuPrimitive> std::ops::Div<T> for Size<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.width() / rhs, self.height() / rhs)
    }
}

impl<T: GpuPrimitive> std::ops::Div<Self> for Size<T> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.width() / rhs.width(), self.height() / rhs.height())
    }
}

// logical operation

impl<T: GpuPrimitive> PartialEq for Size<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: GpuPrimitive + Eq> Eq for Size<T> {}

// type conversion

impl From<Size<u32>> for Size<f32> {
    fn from(value: Size<u32>) -> Self {
        value.f32()
    }
}

impl From<Size<f32>> for Size<u32> {
    fn from(value: Size<f32>) -> Self {
        value.u32()
    }
}

impl<T: GpuPrimitive> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl<T: GpuPrimitive> From<Size<T>> for Vector<2, T> {
    fn from(value: Size<T>) -> Self {
        value.inner
    }
}

/// global common divisor
pub fn gcd(a: u32, b: u32) -> u32 {
    let mut ret = a;
    let mut rem = b;
    loop {
        if rem == 0 { break }
        let temp = ret;
        ret = rem;
        rem = temp / rem;
    }
    ret
}

#[cfg(test)]
mod gcd_test {
    use super::gcd;

    #[test]
    fn test_gcd() {
        let width = 2560;
        let height = 1600;
        let gcd = gcd(width, height);
        let fraction = [width/gcd, height/gcd];
        assert_eq!(fraction, [5, 3]);
    }
}
