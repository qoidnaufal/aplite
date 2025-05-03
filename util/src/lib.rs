mod vector2;
mod vector3;
mod vector4;
mod size;
mod matrix;
mod fraction;

pub use vector2::Vector2;
pub use vector3::Vector3;
pub use vector4::Vector4;
pub use size::Size;
pub use matrix::Matrix;
pub use fraction::Fraction;

pub type Matrix2x2 = Matrix<Vector2<f32>, 2>;
pub type Matrix4x4 = Matrix<Vector4<f32>, 4>;

pub fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

pub(crate) fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        return a;
    }
    gcd(b, a % b)
}
