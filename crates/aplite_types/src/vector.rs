#[repr(C, align(8))]
#[derive(Default, Debug, Clone, Copy)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2u {
    pub x: u32,
    pub y: u32,
}

pub fn vec2f(x: f32, y: f32) -> Vec2f {
    Vec2f { x, y }
}

pub fn vec2u(x: u32, y: u32) -> Vec2u {
    Vec2u { x, y }
}

/*
#########################################################
#                                                       #
#                      impl Vec2f                       #
#                                                       #
#########################################################
*/

impl Vec2f {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// same x & y value
    #[inline(always)]
    pub const fn splat(val: f32) -> Self {
        Self::new(val, val)
    }

    #[inline(always)]
    pub const fn from_array(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }

    #[inline(always)]
    pub const fn array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    #[inline(always)]
    pub const fn point(self) -> crate::point::Point {
        crate::point::Point::new(self.x, self.y)
    }

    #[inline(always)]
    pub fn vec2u(self) -> Vec2u {
        Vec2u::new(self.x.round() as u32, self.y.round() as u32)
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

    /// turn [`Vec2f`] { x, y } into [`Vec2f`] { 1/x, 1/y }
    #[inline(always)]
    pub fn recip(self) -> Self {
        Self {
            x: self.x.recip(),
            y: self.y.recip(),
        }
    }

    /// cross product vector multiplication
    #[inline(always)]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    /// dot product vector multiplication
    #[inline(always)]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// determinant of 2 vectors
    #[inline(always)]
    pub fn det(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// euclidian distance between 2 vectors
    #[inline(always)]
    pub fn dist_euclid(self, other: Self) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        (x.powi(2) + y.powi(2)).sqrt()
    }

    /// the magnitude of a vector, or the displacement from (0.0, 0.0)
    #[inline(always)]
    pub fn length(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// in radians in 4 quadrants (360)
    #[inline(always)]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// in radians in 4 quadrants (360)
    #[inline(always)]
    pub fn angle2(self, other: Self) -> f32 {
        self.det(other).atan2(self.dot(other))
    }
}

/*
#########################################################
#                                                       #
#                      impl Vec2u                       #
#                                                       #
#########################################################
*/

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
    pub fn point(self) -> crate::point::Point {
        crate::point::Point::new(self.x as f32, self.y as f32)
    }

    #[inline(always)]
    pub fn vec2f(self) -> Vec2f {
        Vec2f::new(self.x as f32, self.y as f32)
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
}

/*
#########################################################
#                                                       #
#                      Vec2f From                       #
#                                                       #
#########################################################
*/

impl From<f32> for Vec2f {
    fn from(val: f32) -> Self {
        Self::splat(val)
    }
}

impl From<u32> for Vec2f {
    fn from(val: u32) -> Self {
        Self::splat(val as f32)
    }
}

impl From<[f32; 2]> for Vec2f {
    fn from(arr: [f32; 2]) -> Self {
        Self::from_array(arr)
    }
}

impl From<[u32; 2]> for Vec2f {
    fn from(arr: [u32; 2]) -> Self {
        Self::from_array(arr.map(|n| n as f32))
    }
}

impl From<(f32, f32)> for Vec2f {
    fn from(tuple: (f32, f32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl From<(u32, u32)> for Vec2f {
    fn from(tuple: (u32, u32)) -> Self {
        Self::new(tuple.0 as f32, tuple.1 as f32)
    }
}

impl From<Vec2u> for Vec2f {
    fn from(value: Vec2u) -> Self {
        value.vec2f()
    }
}

impl From<crate::point::Point> for Vec2f {
    fn from(value: crate::point::Point) -> Self {
        value.vec2f()
    }
}

/*
#########################################################
#                                                       #
#                       Vec2f Ops                       #
#                                                       #
#########################################################
*/

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

impl std::ops::Sub<[f32; 2]> for Vec2f {
    type Output = Self;

    fn sub(self, rhs: [f32; 2]) -> Self::Output {
        Self::new(self.x - rhs[0], self.y - rhs[1])
    }
}

impl std::ops::Mul<f32> for Vec2f {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self *= rhs;
        self
    }
}

impl std::ops::MulAssign<f32> for Vec2f {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl std::ops::Div<f32> for Vec2f {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl std::ops::DivAssign<f32> for Vec2f {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl PartialEq for Vec2f {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

/*
#########################################################
#                                                       #
#                      Vec2u From                       #
#                                                       #
#########################################################
*/

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
