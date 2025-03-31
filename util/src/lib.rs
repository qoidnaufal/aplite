mod vector2;
mod vector3;
mod vector4;
mod size;
mod matrix;

pub use vector2::Vector2;
pub use vector3::Vector3;
pub use vector4::Vector4;
pub use size::Size;
pub use matrix::Matrix;

pub type Matrix2x2 = Matrix<Vector2<f32>, 2>;
pub type Matrix4x4 = Matrix<Vector4<f32>, 4>;

pub fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

pub fn cast_slice<SRC: Sized, DST: Sized>(src: &[SRC]) -> &[DST] {
    let len = size_of_val(src);
    unsafe { core::slice::from_raw_parts(src.as_ptr() as *const DST, len) }
}
