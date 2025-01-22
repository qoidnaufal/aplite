use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color<Container, T> {
    data: Vec<T>,
    _phatom: PhantomData<Container>,
}

// impl Color<Rgba<u8>, u8> {
//     pub fn r(&mut self) -> &mut u8 {
//         &mut self[0]
//     }

//     pub fn g(&mut self) -> &mut u8 {
//         &mut self[1]
//     }

//     pub fn b(&mut self) -> &mut u8 {
//         &mut self[2]
//     }
// }

impl<Container, T> std::ops::Index<usize> for Color<Container, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<Container, T> std::ops::IndexMut<usize> for Color<Container, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl std::ops::Deref for Color<Rgba<u8>, u8> {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<Rgb<T>> for Color<Rgb<T>, T> {
    fn from(rgb: Rgb<T>) -> Self {
        Self {
            data: vec![rgb.r, rgb.g, rgb.b],
            _phatom: PhantomData,
        }
    }
}

impl<T> From<Rgba<T>> for Color<Rgba<T>, T> {
    fn from(rgba: Rgba<T>) -> Self {
        Self {
            data: vec![rgba.r, rgba.g, rgba.b, rgba.a],
            _phatom: PhantomData,
        }
    }
}

impl From<Rgb<u8>> for Color<Rgba<u8>, u8> {
    fn from(rgb: Rgb<u8>) -> Self {
        Self {
            data: vec![rgb.r, rgb.g, rgb.b, u8::MAX],
            _phatom: PhantomData,
        }
    }
}

impl From<&[u8]> for Color<Rgba<u8>, u8> {
    fn from(value: &[u8]) -> Self {
        Self {
            data: value.to_vec(),
            _phatom: PhantomData,
        }
    }
}

impl From<Color<Rgba<u8>, u8>> for Color<Rgba<f32>, f32> {
    fn from(color: Color<Rgba<u8>, u8>) -> Self {
        let rgba = color.data;
        Self {
            data: vec![
                rgba[0] as f32 / u8::MAX as f32,
                rgba[1] as f32 / u8::MAX as f32,
                rgba[2] as f32 / u8::MAX as f32,
                rgba[3] as f32 / u8::MAX as f32,
            ],
            _phatom: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl Rgb<u8> {
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const DARK_GRAY: Self = Self { r: 33, g: 33, b: 29 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
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

impl From<Rgb<u8>> for wgpu::Color {
    fn from(rgb: Rgb<u8>) -> Self {
        Self {
            r: rgb.r as f64 / u8::MAX as f64,
            g: rgb.g as f64 / u8::MAX as f64,
            b: rgb.b as f64 / u8::MAX as f64,
            a: 1.0,
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

#[derive(Debug, Clone, Copy)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

// impl Rgba<u8> {
//     pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
//     pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
//     pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
//     pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
// }

impl From<Rgba<u8>> for Rgba<f32> {
    fn from(val: Rgba<u8>) -> Self {
        Self {
            r: val.r as f32 / u8::MAX as f32,
            g: val.g as f32 / u8::MAX as f32,
            b: val.b as f32 / u8::MAX as f32,
            a: val.a as f32 / u8::MAX as f32,
        }
    }
}

impl From<Rgba<f32>> for Rgba<u8> {
    fn from(val: Rgba<f32>) -> Self {
        Self {
            r: (val.r * u8::MAX as f32) as u8,
            g: (val.g * u8::MAX as f32) as u8,
            b: (val.b * u8::MAX as f32) as u8,
            a: (val.a * u8::MAX as f32) as u8,
        }
    }
}

impl PartialEq for Rgba<u8> {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
            && self.a == other.a
    }
}

impl PartialEq for Rgba<f32> {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
            && self.a == other.a
    }
}

