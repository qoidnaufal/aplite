use std::marker::PhantomData;

use util::Size;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixel<Container> {
    dimensions: Size<u32>,
    data: Vec<u8>,
    _phatom: PhantomData<Container>,
}

impl Pixel<Rgba<u8>> {
    pub fn new(dimensions: impl Into<Size<u32>>, data: &[u8]) -> Self {
        Self {
            dimensions: dimensions.into(),
            data: data.to_vec(),
            _phatom: PhantomData,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.dimensions.width as f32 / self.dimensions.height as f32
    }

    pub fn dimensions(&self) -> Size<u32> {
        self.dimensions
    }
}

impl std::ops::Deref for Pixel<Rgba<u8>> {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<Rgb<u8>> for Pixel<Rgb<u8>> {
    fn from(rgb: Rgb<u8>) -> Self {
        Self {
            dimensions: Size::new(1, 1),
            data: vec![rgb.r, rgb.g, rgb.b],
            _phatom: PhantomData,
        }
    }
}

impl From<Rgb<u8>> for Pixel<Rgba<u8>> {
    fn from(rgb: Rgb<u8>) -> Self {
        Self {
            dimensions: Size::new(1, 1),
            data: vec![rgb.r, rgb.g, rgb.b, u8::MAX],
            _phatom: PhantomData,
        }
    }
}

impl From<Rgba<u8>> for Pixel<Rgba<u8>> {
    fn from(rgba: Rgba<u8>) -> Self {
        Self {
            dimensions: Size::new(1, 1),
            data: vec![rgba.r, rgba.g, rgba.b, rgba.a],
            _phatom: PhantomData,
        }
    }
}

#[repr(C)]
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
    pub const DARK_GRAY: Self = Self { r: 30, g: 30, b: 30 };
}

impl From<Rgb<u8>> for wgpu::Color {
    fn from(rgb8: Rgb<u8>) -> Self {
        Self {
            r: rgb8.r as f64 / u8::MAX as f64,
            g: rgb8.g as f64 / u8::MAX as f64,
            b: rgb8.b as f64 / u8::MAX as f64,
            a: 1.0,
        }
    }
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

impl From<Rgba<f32>> for Rgb<u8> {
    fn from(rgba_f32: Rgba<f32>) -> Self {
        let rgba_u8: Rgba<u8> = rgba_f32.into();
        Self {
            r: rgba_u8.r,
            g: rgba_u8.g,
            b: rgba_u8.b,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl Rgba<u8> {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0, a: 255 };
    pub const DARK_GRAY: Self = Self { r: 30, g: 30, b: 30, a: 255 };
}

impl From<Rgba<u8>> for u32 {
    fn from(rgba: Rgba<u8>) -> Self {
        ((rgba.r as u32) << 24)
        | ((rgba.g as u32) << 16)
        | ((rgba.b as u32) << 8)
        | (rgba.a as u32)
    }
}

impl From<u32> for Rgba<u8> {
    fn from(num: u32) -> Self {
        let r = (num >> 24) as u8;
        let g = ((num >> 16) & 0xFF) as u8;
        let b = ((num >> 8) & 0xFF) as u8;
        let a = (num & 0xFF) as u8;
        Self { r, g, b, a }
    }
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

impl From<Rgb<u8>> for Rgba<u8> {
    fn from(value: Rgb<u8>) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: u8::MAX,
        }
    }
}

impl From<Rgb<u8>> for Rgba<f32> {
    fn from(value: Rgb<u8>) -> Self {
        let rgba: Rgba<u8> = value.into();
        rgba.into()
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

