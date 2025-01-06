#[derive(Debug, Clone, Copy)]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl<T: Default> Default for Rgb<T> {
    fn default() -> Self {
        Self {
            r: T::default(),
            g: T::default(),
            b: T::default(),
        }
    }
}

impl Rgb<u8> {
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
}

impl From<Rgb<u8>> for Rgb<f32> {
    fn from(val: Rgb<u8>) -> Self {
        Self {
            r: val.r as f32 / u8::MAX as f32,
            g: val.g as f32 / u8::MAX as f32,
            b: val.b as f32 / u8::MAX as f32,
        }
    }
}

impl From<Rgb<f32>> for Rgb<u8> {
    fn from(val: Rgb<f32>) -> Self {
        Self {
            r: (val.r * u8::MAX as f32) as u8,
            g: (val.g * u8::MAX as f32) as u8,
            b: (val.b * u8::MAX as f32) as u8,
        }
    }
}

impl PartialEq for Rgb<u8> {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
    }
}

impl PartialEq for Rgb<f32> {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
    }
}

