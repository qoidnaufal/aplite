/// accept 1 << 16 bytes of color, alpha will always be 255
pub const fn rgb(val: u32) -> Color {
    let r = (val >> 16) as u8;
    let g = ((val >> 8) & 0xFF) as u8;
    let b = (val & 0xFF) as u8;

    Color { r, g, b, a: 255 }
}

/// accept 1 << 24 bytes of color
pub const fn rgba(val: u32) -> Color {
    let r = (val >> 24) as u8;
    let g = ((val >> 16) & 0xFF) as u8;
    let b = ((val >> 8) & 0xFF) as u8;
    let a = (val & 0xFF) as u8;

    Color { r, g, b, a }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
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
    pub fn with_alpha_u8(self, a: u8) -> Self {
        Self {
            a,
            ..self
        }
    }

    #[inline(always)]
    pub fn with_alpha_f32(self, a: f32) -> Self {
        Self {
            a: (a.clamp(0.0, 1.0) * u8::MAX as f32) as u8,
            ..self
        }
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

pub mod theme {
    pub use super::gruvbox_dark;
    pub use super::basic;
}

pub mod basic {
    use super::{Color, rgb, rgba};

    pub const TRANSPARENT: Color = rgba(0x00000000);
    pub const BLACK: Color = rgb(0x000000);
    pub const WHITE: Color = rgb(0xffffff);

    pub const RED: Color = rgb(0xff0000);
    pub const GREEN: Color = rgb(0x00ff00);
    pub const BLUE: Color = rgb(0x0000ff);

    pub const YELLOW: Color = rgb(0xffff00);
}

pub mod gruvbox_dark {
    use super::{Color, rgb};

    pub const BG_0: Color = rgb(0x282828);
    pub const BG_H: Color = rgb(0x1d2021);
    pub const BG_S: Color = rgb(0x32302f);

    pub const FG_0: Color = rgb(0xfbf1c7);
    pub const FG_1: Color = rgb(0xebdbb2);

    pub const RED_0: Color = rgb(0xcc241d);
    pub const RED_1: Color = rgb(0xfb4934);

    pub const GREEN_0: Color = rgb(0x98971a);
    pub const GREEN_1: Color = rgb(0xb8bb26);

    pub const YELLOW_0: Color = rgb(0xd79921);
    pub const YELLOW_1: Color = rgb(0xfabd2f);

    pub const BLUE_0: Color = rgb(0x458588);
    pub const BLUE_1: Color = rgb(0x83a598);

    pub const PURPLE_0: Color = rgb(0xb16286);
    pub const PURPLE_1: Color = rgb(0xd3869b);

    pub const AQUA_0: Color = rgb(0x689d6a);
    pub const AQUA_1: Color = rgb(0x8ec07c);

    pub const ORANGE_0: Color = rgb(0xd65d0e);
    pub const ORANGE_1: Color = rgb(0xfe8019);
}
