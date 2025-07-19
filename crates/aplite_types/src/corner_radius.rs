#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy)]
pub struct CornerRadius {
    pub tl: f32,
    pub bl: f32,
    pub br: f32,
    pub tr: f32
}

impl CornerRadius {
    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn new(tl: f32, bl: f32, br: f32, tr: f32) -> Self {
        Self { tl, bl, br, tr }
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn splat(r: f32) -> Self {
        Self::new(r, r, r, r)
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn set_each(&mut self, tl: f32, bl: f32, br: f32, tr: f32) {
        self.tl = tl;
        self.bl = bl;
        self.br = br;
        self.tr = tr;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    #[inline(always)]
    pub const fn set_all(&mut self, val: f32) {
        self.tl = val;
        self.bl = val;
        self.br = val;
        self.tr = val;
    }
}

impl From<u32> for CornerRadius {
    fn from(val: u32) -> Self {
        Self::splat(val as f32)
    }
}

impl From<f32> for CornerRadius {
    fn from(value: f32) -> Self {
        Self::splat(value)
    }
}
