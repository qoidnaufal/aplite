#[inline(always)]
 pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
    Rgba::new(r, g, b, a)
}

/// value must be between 0.0 and 1.0
#[inline(always)]
pub const fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Rgba<f32> {
    Rgba::new(r, g, b, a)
}

#[inline(always)]
pub fn rgba_hex(hex: &str) -> Rgba<u8> {
    Rgba::from_hex(hex)
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T: Clone + Copy> Rgba<T> {
    #[inline(always)]
    pub const fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> [T; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Rgba<u8> {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);
    pub const PURPLE: Self = Self::new(111, 72, 234, 255);
    pub const LIGHT_GRAY: Self = Rgba::new(69, 69, 69, 255);
    pub const DARK_GRAY: Self = Self::new(30, 30, 30, 255);
    pub const DARK_GREEN: Self = Self::new(10, 30, 15, 255);

    pub fn from_hex(hex: &str) -> Self {
        let (r, g, b, a) = parse_hex(hex);
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub const fn from_u32(val: u32) -> Self {
        let r = (val >> 24) as u8;
        let g = ((val >> 16) & 0xFF) as u8;
        let b = ((val >> 8) & 0xFF) as u8;
        let a = (val & 0xFF) as u8;
        Self::new(r, g, b, a)
    }

    #[inline(always)]
    pub const fn into_u32(self) -> u32 {
        ((self.r as u32) << 24)
        | ((self.g as u32) << 16)
        | ((self.b as u32) << 8)
        | (self.a as u32)
    }

    pub fn f32(self) -> Rgba<f32> {
        self.into()
    }
}

impl Rgba<f32> {
    pub fn u8(self) -> Rgba<u8> {
        self.into()
    }
}

impl From<Rgba<u8>> for u32 {
    fn from(rgba: Rgba<u8>) -> Self {
        rgba.into_u32()
    }
}

impl From<u32> for Rgba<u8> {
    fn from(num: u32) -> Self {
        Self::from_u32(num)
    }
}

impl From<Rgba<u8>> for Rgba<f32> {
    fn from(val: Rgba<u8>) -> Self {
        Self::new(
            val.r as f32 / u8::MAX as f32,
            val.g as f32 / u8::MAX as f32,
            val.b as f32 / u8::MAX as f32,
            val.a as f32 / u8::MAX as f32,
        )
    }
}

impl From<Rgba<f32>> for Rgba<u8> {
    fn from(val: Rgba<f32>) -> Self {
        Self::new(
            (val.r * u8::MAX as f32).round() as u8,
            (val.g * u8::MAX as f32).round() as u8,
            (val.b * u8::MAX as f32).round() as u8,
            (val.a * u8::MAX as f32).round() as u8,
        )
    }
}

impl<T: PartialEq> PartialEq for Rgba<T> {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
            && self.a == other.a
    }
}

impl Eq for Rgba<u8> {}

#[inline(always)]
fn parse_hex(hex: &str) -> (u8, u8, u8, u8) {
    assert!(hex.get(..1).unwrap() == "#", "input doesn't start with #");
    assert!(hex.get(1..).is_some_and(|s| s.len() == 8), "invalid input length, expected 8");

    let mut buf = [0; 8];

    hex.chars()
        .skip(1)
        .enumerate()
        .for_each(|(i, c)| {
            match c.to_digit(16) {
                Some(num) => buf[i] = num as u8,
                None => panic!("invalid char {}", c),
            }
        });

    let r = buf[0] * 16 + buf[1];
    let g = buf[2] * 16 + buf[3];
    let b = buf[4] * 16 + buf[5];
    let a = buf[6] * 16 + buf[7];

    (r, g, b, a)
}
