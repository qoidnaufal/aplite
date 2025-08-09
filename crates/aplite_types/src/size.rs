use crate::fraction::Fraction;

/// corresponds to [`winit::dpi::LogicalSize<T>`]
#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    #[inline(always)]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[inline(always)]
    pub const fn area(&self) -> f32 { self.width * self.height }

    #[inline(always)]
    pub fn diagonal(&self) -> f32 {
        (self.width.powi(2) + self.height.powi(2)).sqrt()
    }

    #[inline(always)]
    pub const fn min(self, other: Self) -> Self {
        Self::new(
            self.width.min(other.width),
            self.height.min(other.height)
        )
    }

    #[inline(always)]
    pub const fn max(self, other: Self) -> Self {
        Self::new(
            self.width.max(other.width),
            self.height.max(other.height)
        )
    }

    #[inline(always)]
    pub const fn clamp(self, start: Self, end: Self) -> Self {
        self.max(start).min(end)
    }

    pub fn adjust_on_min_constraints(self, min_width: Option<f32>, min_height: Option<f32>) -> Self {
        let width = min_width.map(|w| self.width.max(w)).unwrap_or(self.width);
        let height = min_height.map(|h| self.height.max(h)).unwrap_or(self.height);
        Self::new(width, height)
    }

    pub fn adjust_on_max_constraints(self, max_width: Option<f32>, max_height: Option<f32>) -> Self {
        let width = max_width.map(|w| self.width.min(w)).unwrap_or(self.width);
        let height = max_height.map(|h| self.height.min(h)).unwrap_or(self.height);
        Self::new(width, height)
    }
    
    pub fn adjust_width_aspect_ratio(&mut self, aspect_ratio: Fraction) {
        self.width = self.height * aspect_ratio
    }

    pub fn adjust_height_aspect_ratio(&mut self, aspect_ratio: Fraction) {
        self.height = self.width / aspect_ratio
    }

    pub fn aspect_ratio(&self) -> Fraction {
        let gcd = gcd(self.width, self.height);
        Fraction::new(self.width / gcd, self.height / gcd)
    }

    pub fn rect(self) -> crate::Rect {
        crate::Rect::from_size(self)
    }
}

// arithmetic operation

impl std::ops::Mul<f32> for Size {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.width * rhs, self.height * rhs)
    }
}

impl std::ops::MulAssign<f32> for Size {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

impl std::ops::Div<f32> for Size {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.width / rhs, self.height / rhs)
    }
}

impl std::ops::Div<Self> for Size {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.width / rhs.width, self.height / rhs.height)
    }
}

// logical operation

impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
    }
}

impl Eq for Size {}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.area().partial_cmp(&other.area())
    }
}

impl Ord for Size {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.area().total_cmp(&other.area())
    }
}

// type conversion

impl From<(u32, u32)> for Size {
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0 as f32, value.1 as f32)
    }
}

impl From<(f32, f32)> for Size {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

/// global common divisor
pub fn gcd(a: f32, b: f32) -> f32 {
    let mut ret = a;
    let mut rem = b;
    loop {
        if rem == 0.0 { break }
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
        let width = 2560.;
        let height = 1600.;
        let gcd = gcd(width, height);
        let fraction = [width/gcd, height/gcd];
        assert_eq!(fraction, [5., 3.]);
    }
}
