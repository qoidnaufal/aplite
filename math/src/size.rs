#[derive(Debug,Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl Size<f32> {
    pub fn scale(&self, other: Self) -> Self {
        Self {
            width: self.width / other.width,
            height: self.height / other.height,
        }
    }
}

impl From<Size<u32>> for Size<f32> {
    fn from(value: Size<u32>) -> Self {
        Self  {
            width: value.width as _,
            height: value.height as _,
        }
    }
}

impl<T> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl PartialEq for Size<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl Eq for Size<u32> {}

impl std::ops::Div<f32> for Size<f32> {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}
