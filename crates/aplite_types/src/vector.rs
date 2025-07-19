#[repr(C, align(8))]
#[derive(Default, Debug, Clone, Copy)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn from_array(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }

    #[inline(always)]
    pub const fn splat(val: f32) -> Self {
        Self::new(val, val)
    }

    #[inline(always)]
    pub const fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y)
        )
    }

    #[inline(always)]
    pub const fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y)
        )
    }

    #[inline(always)]
    pub const fn clamp(self, start: Self, end: Self) -> Self {
        self.max(start).min(end)
    }

    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline(always)]
    pub fn to_vec2u(self) -> Vec2u {
        Vec2u::new(self.x.round() as u32, self.y.round() as u32)
    }
}

impl From<(f32, f32)> for Vec2f {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl std::ops::Add for Vec2f {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Add<[f32; 2]> for Vec2f {
    type Output = Self;
    fn add(self, rhs: [f32; 2]) -> Self::Output {
        Self::new(self.x + rhs[0], self.y + rhs[1])
    }
}

impl std::ops::Sub for Vec2f {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl PartialEq for Vec2f {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2u {
    pub x: u32,
    pub y: u32,
}

impl Vec2u {
    #[inline(always)]
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn splat(val: u32) -> Self {
        Self::new(val, val)
    }

    #[inline(always)]
    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y)
        )
    }

    #[inline(always)]
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y)
        )
    }

    #[inline(always)]
    pub fn clamp(self, start: Self, end: Self) -> Self {
        self.max(start).min(end)
    }

    #[inline(always)]
    pub fn to_vec2f(self) -> Vec2f {
        Vec2f::new(self.x as f32, self.y as f32)
    }
}

impl std::ops::Mul<u32> for Vec2u {
    type Output = Self;
    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Div<u32> for Vec2u {
    type Output = Self;
    fn div(self, rhs: u32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl PartialEq for Vec2u {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl PartialOrd for Vec2u {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.x
            .partial_cmp(&other.x)
            .map(|ord| ord.then(self.y.cmp(&other.y)))
    }
}
