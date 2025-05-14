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
    use super::{Vector2, Size, GpuPrimitive, NumDebugger};

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

    impl<T: GpuPrimitive> Rect<T> {
        pub const fn new(pos: Vector2<T>, size: Size<T>) -> Self {
            Self { pos, size }
        }

        #[inline(always)]
        pub const fn pos(&self) -> Vector2<T> { self.pos }

        #[inline(always)]
        pub const fn size(&self) -> Size<T> { self.size }

        #[inline(always)]
        pub fn pos_mut(&mut self) -> &mut Vector2<T> { &mut self.pos }

        #[inline(always)]
        pub fn size_mut(&mut self) -> &mut Size<T> { &mut self.size }

        #[inline(always)]
        pub fn set_pos(&mut self, pos: Vector2<T>) { self.pos = pos }

        #[inline(always)]
        pub fn set_size(&mut self, size: Size<T>) { self.size = size }

        #[inline(always)]
        pub const fn left(&self) -> T { self.pos.x() }

        #[inline(always)]
        pub fn bottom(&self) -> T { self.pos.x() + self.size.height() }

        #[inline(always)]
        pub fn right(&self) -> T { self.pos.x() + self.size.width() }

        #[inline(always)]
        pub const fn top(&self) -> T { self.pos.y() }
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
