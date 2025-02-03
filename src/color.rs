use std::marker::PhantomData;

use util::Size;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color<Container, T> {
    dimensions: Size<u32>,
    data: Vec<T>,
    _phatom: PhantomData<Container>,
}

impl Color<Rgba<u8>, u8> {
    pub fn new(dimensions: impl Into<Size<u32>>, data: &[u8]) -> Self {
        Self {
            dimensions: dimensions.into(),
            data: data.to_vec(),
            _phatom: PhantomData,
        }
    }

    pub fn dimensions(&self) -> Size<u32> {
        self.dimensions
    }
}

impl<T> std::ops::Deref for Color<Rgba<T>, T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<Rgb<u8>> for Color<Rgb<u8>, u8> {
    fn from(rgb: Rgb<u8>) -> Self {
        Self {
            dimensions: Size::new(1, 1),
            data: vec![rgb.r, rgb.g, rgb.b],
            _phatom: PhantomData,
        }
    }
}

impl From<Rgb<u8>> for Color<Rgba<u8>, u8> {
    fn from(rgb: Rgb<u8>) -> Self {
        Self {
            dimensions: Size::new(1, 1),
            data: vec![rgb.r, rgb.g, rgb.b, u8::MAX],
            _phatom: PhantomData,
        }
    }
}

impl From<Rgba<u8>> for Color<Rgba<u8>, u8> {
    fn from(rgba: Rgba<u8>) -> Self {
        Self {
            dimensions: Size::new(1, 1),
            data: vec![rgba.r, rgba.g, rgba.b, rgba.a],
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
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0 };
    pub const GRAY: Self = Self { r: 56, g: 57, b: 58 };
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

