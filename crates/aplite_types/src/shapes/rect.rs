use crate::fraction::Fraction;
use crate::size::Size;
use crate::vector::Vec2f;
use crate::point::Point;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Calculate a [`Rect`] from the two given points
    #[inline(always)]
    pub fn from_vec2f(p1: Vec2f, p2: Vec2f) -> Self {
        let origin = p1.min(p2);
        let size = p1.max(p2) - origin;
        Self {
            x: origin.x,
            y: origin.y,
            width: size.x,
            height: size.y,
        }
    }

    #[inline(always)]
    pub const fn from_vec2f_size(vec2f: Vec2f, size: Size) -> Self {
        Self::new(vec2f.x, vec2f.y, size.width, size.height)
    }

    #[inline(always)]
    pub const fn from_point_size(point: Point, size: Size) -> Self {
        Self::new(point.x, point.y, size.width, size.height)
    }

    #[inline(always)]
    pub const fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }

    #[inline(always)]
    pub const fn vec2f(&self) -> Vec2f {
        Vec2f::new(self.x, self.y)
    }

    #[inline(always)]
    pub const fn point(&self) -> crate::Point {
        crate::Point::new(self.x, self.y)
    }

    #[inline(always)]
    pub const fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    #[inline(always)]
    pub const fn set_pos(&mut self, pos: Vec2f) {
        self.x = pos.x;
        self.y = pos.y;
    }

    #[inline(always)]
    pub const fn set_size(&mut self, size: Size) {
        self.width = size.width;
        self.height = size.height
    }

    #[inline(always)]
    pub const fn max_x(&self) -> f32 { self.x + self.width }

    #[inline(always)]
    pub const fn max_y(&self) -> f32 { self.y + self.height }

    #[inline(always)]
    pub const fn center_x(&self) -> f32 { self.x + self.width / 2. }

    #[inline(always)]
    pub const fn center_y(&self) -> f32 { self.y + self.height / 2. }

    #[inline(always)]
    pub const fn area(&self) -> f32 {
        self.width * self.height
    }

    #[inline(always)]
    pub fn contains(&self, p: &Vec2f) -> bool {
        (self.x..self.max_x()).contains(&p.x)
            && (self.y..self.max_y()).contains(&p.y)
    }

    #[inline(always)]
    pub fn adjust_width(&mut self, aspect_ratio: Fraction) {
        self.width = self.height * aspect_ratio
    }

    #[inline(always)]
    pub fn adjust_height(&mut self, aspect_ratio: Fraction) {
        self.height = self.width / aspect_ratio
    }
}

impl From<(Vec2f, Vec2f)> for Rect {
    fn from((v0, v1): (Vec2f, Vec2f)) -> Self {
        Self::from_vec2f(v0, v1)
    }
}

impl From<[Vec2f; 2]> for Rect {
    fn from([v0, v1]: [Vec2f; 2]) -> Self {
        Self::from_vec2f(v0, v1)
    }
}

impl From<&[Vec2f]> for Rect {
    fn from(slice: &[Vec2f]) -> Self {
        Self::from_vec2f(slice[0], slice[1])
    }
}

impl From<(Vec2f, Size)> for Rect {
    fn from((vec2f, size): (Vec2f, Size)) -> Self {
        Self::from_vec2f_size(vec2f, size)
    }
}

impl PartialEq for Rect {
    fn eq(&self, other: &Self) -> bool {
        self.vec2f().eq(&other.vec2f())
            && self.size().eq(&other.size())
    }
}

impl Eq for Rect {}

impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size().cmp(&other.size())
    }
}
