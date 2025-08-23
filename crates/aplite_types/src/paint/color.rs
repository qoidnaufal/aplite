 pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba::new(r, g, b, a)
}

pub const fn rgba32(val: u32) -> Rgba {
    Rgba::from_u32(val)
}

pub fn rgba_hex(hex: &str) -> Rgba {
    Rgba::from_hex(hex)
}

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
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

    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub fn from_hex(hex: &str) -> Self {
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

        Self {
            r: buf[0] * 16 + buf[1],
            g: buf[2] * 16 + buf[3],
            b: buf[4] * 16 + buf[5],
            a: buf[6] * 16 + buf[7],
        }
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
    pub const fn pack_u32(self) -> u32 {
        ((self.r as u32) << 24)
        | ((self.g as u32) << 16)
        | ((self.b as u32) << 8)
        | (self.a as u32)
    }

    #[inline(always)]
    pub fn as_slice(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<Rgba> for u32 {
    fn from(rgba: Rgba) -> Self {
        rgba.pack_u32()
    }
}

impl From<u32> for Rgba {
    fn from(num: u32) -> Self {
        Self::from_u32(num)
    }
}

impl PartialEq for Rgba {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
            && self.a == other.a
    }
}

impl Eq for Rgba {}
