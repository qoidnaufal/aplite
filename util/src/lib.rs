mod vector2;
mod vector3;
mod size;
mod matrix;

pub use vector2::Vector2;
pub use vector3::Vector3;
pub use size::Size;
pub use matrix::{Matrix, Vector4};

pub type Matrix4x4 = Matrix<Vector4<f32>, 4>;

pub fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

pub fn cast_slice<SRC: Sized, DST: Sized>(src: &[SRC]) -> &[DST] {
    let len = src.len() * size_of::<SRC>();
    unsafe { core::slice::from_raw_parts(src.as_ptr() as *const DST, len) }
}
