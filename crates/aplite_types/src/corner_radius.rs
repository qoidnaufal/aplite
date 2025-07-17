use crate::num_traits::GpuPrimitive;
use crate::vector::Vector;

#[repr(C, align(16))]
#[derive(Default, Clone, Copy)]
pub struct CornerRadius<T: GpuPrimitive> {
    inner: Vector<4, T>
}

impl<T: GpuPrimitive> CornerRadius<T> {
    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn new_each(tl: T, bl: T, br: T, tr: T) -> Self {
        Self { inner: Vector::new_from_array([tl, bl, br, tr]) }
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn new_all(r: T) -> Self {
        Self { inner: Vector::new_from_array([r; 4]) }
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn set_each(&mut self, tl: T, bl: T, br: T, tr: T) {
        self.inner.inner[0] = tl;
        self.inner.inner[1] = bl;
        self.inner.inner[2] = br;
        self.inner.inner[3] = tr;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn set_all(&mut self, val: T) {
        self.inner.inner[0] = val;
        self.inner.inner[1] = val;
        self.inner.inner[2] = val;
        self.inner.inner[3] = val;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn set_top_left(&mut self, val: T) {
        self.inner.inner[0] = val;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn set_bot_left(&mut self, val: T) {
        self.inner.inner[1] = val;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn set_bot_right(&mut self, val: T) {
        self.inner.inner[2] = val;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn set_top_right(&mut self, val: T) {
        self.inner.inner[3] = val;
    }
}

impl From<u32> for CornerRadius<u32> {
    fn from(val: u32) -> Self {
        Self::new_all(val)
    }
}

impl From<f32> for CornerRadius<f32> {
    fn from(value: f32) -> Self {
        Self::new_all(value)
    }
}

impl<T: GpuPrimitive> std::fmt::Debug for CornerRadius<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CornerRadius")
            .field("tl", &self.inner[0])
            .field("bl", &self.inner[1])
            .field("br", &self.inner[2])
            .field("tr", &self.inner[3])
            .finish()
    }
}
