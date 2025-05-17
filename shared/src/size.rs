use winit::dpi::LogicalSize;

use super::{Fraction, GpuPrimitive, NumDebugger, Vector2};

fn gcd(a: u32, b: u32) -> u32 {
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

/// corresponds to [`winit::dpi::LogicalSize<T>`]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Size<T: GpuPrimitive> {
    inner: Vector2<T>
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
            inner: Vector2::default()
        }
    }
}

impl<T: GpuPrimitive> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { inner: Vector2::new(width, height) }
    }

    #[inline(always)]
    pub const fn width(&self) -> T { self.inner.x() }

    #[inline(always)]
    pub const fn height(&self) -> T { self.inner.y() }

    pub fn set_width(&mut self, val: T) { self.inner.set_x(val) }

    pub fn set_height(&mut self, val: T) { self.inner.set_y(val) }

    pub fn add_width(&mut self, val: T) { self.inner.add_x(val) }

    pub fn add_height(&mut self, val: T) { self.inner.add_y(val) }

    pub fn mul_width(&mut self, val: T) { self.inner.mul_x(val) }

    pub fn mul_height(&mut self, val: T) { self.inner.mul_y(val) }
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

impl Size<u32> {
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

impl From<Size<u32>> for LogicalSize<u32> {
    fn from(value: Size<u32>) -> Self {
        Self::new(value.width(), value.height())
    }
}

impl From<LogicalSize<u32>> for Size<u32> {
    fn from(value: LogicalSize<u32>) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<Size<u32>> for Size<f32> {
    fn from(value: Size<u32>) -> Self {
        Self::new(value.width() as _, value.height() as _)
    }
}

impl From<Size<f32>> for Size<u32> {
    fn from(value: Size<f32>) -> Self {
        Self::new(value.width() as _, value.height() as _)
    }
}

impl<T: GpuPrimitive> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl<T: GpuPrimitive> From<Size<T>> for Vector2<T> {
    fn from(value: Size<T>) -> Self {
        value.inner
    }
}
