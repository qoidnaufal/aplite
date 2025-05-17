mod color;
mod fraction;
mod matrix;
mod size;
mod traits;
mod vector;

pub use traits::*;
pub use size::Size;
pub use matrix::Matrix4x4;
pub use fraction::Fraction;
pub use color::Rgba;
pub use rect::Rect;
pub use vector::{
    Vector2,
    Vector3,
    Vector4,
};

// pub fn tan(x: f32, y: f32) -> f32 {
//     (y / x).abs()
// }

mod rect {
    use super::{Vector2, Size, GpuPrimitive, NumDebugger, Fraction};

    #[derive(Clone, Copy)]
    pub struct Rect<T: GpuPrimitive> {
        pos: Vector2<T>,
        size: Size<T>,
    }

    impl<T: GpuPrimitive + NumDebugger> std::fmt::Debug for Rect<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f, "Rect {{ x: {}, y: {}, width: {}, height: {} }}",
                self.pos.x(), self.pos.y(), self.size.width(), self.size.height()
            )
        }
    }

    impl<T: GpuPrimitive> PartialEq for Rect<T> {
        fn eq(&self, other: &Self) -> bool {
            self.pos == other.pos && self.size == other.size
        }
    }

    impl<T: GpuPrimitive> Eq for Rect<T> {}

    impl<T: GpuPrimitive> Rect<T> {
        pub const fn new(pos: Vector2<T>, size: Size<T>) -> Self {
            Self { pos, size }
        }

        #[inline(always)]
        pub const fn pos(&self) -> Vector2<T> { self.pos }

        #[inline(always)]
        pub fn set_pos(&mut self, pos: Vector2<T>) { self.pos = pos }

        #[inline(always)]
        pub const fn x(&self) -> T { self.pos.x() }

        #[inline(always)]
        pub fn set_x(&mut self, x: T) { self.pos.set_x(x) }

        #[inline(always)]
        pub fn add_x(&mut self, x: T) { self.pos.add_x(x) }

        #[inline(always)]
        pub const fn y(&self) -> T { self.pos.y() }

        #[inline(always)]
        pub fn set_y(&mut self, y: T) { self.pos.set_y(y) }

        #[inline(always)]
        pub fn add_y(&mut self, y: T) { self.pos.add_y(y) }

        #[inline(always)]
        pub const fn size(&self) -> Size<T> { self.size }

        #[inline(always)]
        pub fn set_size(&mut self, size: Size<T>) { self.size = size }

        #[inline(always)]
        pub const fn width(&self) -> T { self.size.width() }

        #[inline(always)]
        pub fn set_width(&mut self, width: T) { self.size.set_width(width) }

        #[inline(always)]
        pub fn add_width(&mut self, width: T) { self.size.add_width(width) }

        #[inline(always)]
        pub const fn height(&self) -> T { self.size.height() }

        #[inline(always)]
        pub fn set_height(&mut self, height: T) { self.size.set_height(height) }

        #[inline(always)]
        pub fn add_height(&mut self, height: T) { self.size.add_height(height) }

        #[inline(always)]
        pub const fn left(&self) -> T { self.pos.x() }

        #[inline(always)]
        pub fn bottom(&self) -> T { self.pos.x() + self.size.height() }

        #[inline(always)]
        pub fn right(&self) -> T { self.pos.x() + self.size.width() }

        #[inline(always)]
        pub const fn top(&self) -> T { self.pos.y() }
    }

    impl Rect<u32> {
        pub fn adjust_width(&mut self, aspect_ratio: Fraction<u32>) {
            self.size.adjust_width(aspect_ratio);
        }

        pub fn adjust_height(&mut self, aspect_ratio: Fraction<u32>) {
            self.size.adjust_height(aspect_ratio);
        }
    }

    impl From<Rect<f32>> for Rect<u32> {
        fn from(value: Rect<f32>) -> Self {
            Self {
                pos: value.pos.into(),
                size: value.size.into(),
            }
        }
    }

    impl From<Rect<u32>> for Rect<f32> {
        fn from(value: Rect<u32>) -> Self {
            Self {
                pos: value.pos.into(),
                size: value.size.into(),
            }
        }
    }
}
