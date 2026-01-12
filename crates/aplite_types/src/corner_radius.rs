#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CornerRadius {
    pub tl: u8,
    pub bl: u8,
    pub br: u8,
    pub tr: u8
}

impl CornerRadius {
    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn new(tl: u8, bl: u8, br: u8, tr: u8) -> Self {
        Self { tl, bl, br, tr }
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn splat(r: u8) -> Self {
        Self::new(r, r, r, r)
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn set_each(&mut self, tl: u8, bl: u8, br: u8, tr: u8) {
        self.tl = tl;
        self.bl = bl;
        self.br = br;
        self.tr = tr;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn set_all(&mut self, val: u8) {
        self.tl = val;
        self.bl = val;
        self.br = val;
        self.tr = val;
    }

    pub fn pack_u32(&self) -> u32 {
        (self.tl as u32) << 24
        | (self.bl as u32) << 16
        | (self.br as u32) << 8
        | (self.tr as u32)
    }
}

impl From<u32> for CornerRadius {
    fn from(val: u32) -> Self {
        Self::splat(val as u8)
    }
}

impl From<u8> for CornerRadius {
    fn from(value: u8) -> Self {
        Self::splat(value)
    }
}
