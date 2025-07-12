use std::ops::{Index, IndexMut};

use crate::vector::{Vector, Vec2f, Vec4f};

/// GPU's mat3x2 is actually a \[[`Vector<2, T>`]; 3\] in CPU
/// # Representation:
/// CPU:     │ GPU:
/// x  ,  y  │ x: x0, x1, x2
/// x0 , y0  │ y: y0, y1, y2
/// x1 , y1  │ 
/// x2 , y2  │ 
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Matrix<const M: usize, const N: usize> {
    inner: [Vector<N, f32>; M]
}

impl<const M: usize, const N: usize> std::fmt::Debug for Matrix<M, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push('\n');
        for n in 0..N {
            let (prefix, suffix) = match n {
                0 => ("x │", "│\n"),
                1 => ("y │", "│\n"),
                2 => ("z │", "│\n"),
                3 => ("w │", "│"),
                _ => panic!()
            };
            s.push_str(prefix);
            for m in 0..M {
                let num = self[m][n];
                if num.is_sign_negative() {
                    s.push_str(format!("{num:0.2} ").as_str());
                } else {
                    s.push_str(format!(" {num:0.2} ").as_str());
                }
            }
            s.push_str(suffix);
        }
        write!(f, "{s}")
    }
}

impl<const M: usize, const N: usize> Index<usize> for Matrix<M, N> {
    type Output = Vector<N, f32>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<const M: usize, const N: usize> IndexMut<usize> for Matrix<M, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<const M: usize, const N: usize> Matrix<M, N> {
    const fn zero() -> Self {
        Self { inner: [ Vector::new_from_array([0.0; N]); M ] }
    }

    #[inline(always)]
    pub const fn data(&self) -> &[Vector<N, f32>] {
        &self.inner
    }

    const fn get_scale(&self) -> (f32, f32) {
        (self.inner[0].inner[0], self.inner[1].inner[1])
    }

    #[inline(always)]
    pub const fn with_translate(mut self, x: f32, y: f32) -> Self {
        self.set_translate(x, y);
        self
    }

    #[inline(always)]
    pub const fn set_translate(&mut self, x: f32, y: f32) {
        self.inner[M - 1].inner[0] = x;
        self.inner[M - 1].inner[1] = y;
    }

    #[inline(always)]
    pub const fn with_scale(mut self, sx: f32, sy: f32) -> Self {
        self.set_scale(sx, sy);
        self
    }

    #[inline(always)]
    pub const fn set_scale(&mut self, sx: f32, sy: f32) {
        self.inner[0].inner[0] = sx;
        self.inner[1].inner[1] = sy;
    }

    #[inline(always)]
    pub const fn mul_scale(&mut self, sx: f32, sy: f32) {
        self.inner[0].inner[0] *= sx;
        self.inner[1].inner[1] *= sy;
    }

    #[inline(always)]
    /// rotation need to be in radians
    pub fn with_scale_and_rotation_rad(mut self, sx: f32, sy: f32, rad: f32) -> Self {
        self.set_scale_and_rotation_rad(sx, sy, rad);
        self
    }

    #[inline(always)]
    /// rotation need to be in radians
    pub fn set_scale_and_rotation_rad(&mut self, sx: f32, sy: f32, rad: f32) {
        self[0].inner[0] = rad.cos() * sx;
        self[0].inner[1] =-rad.sin() * sy;
        self[1].inner[0] = rad.sin() * sx;
        self[1].inner[1] = rad.cos() * sy;
    }

    #[inline(always)]
    /// rotation need to be in degree, will be converted into radians internally
    pub fn with_scale_and_rotation_deg(mut self, sx: f32, sy: f32, rotation: f32) -> Self {
        self.set_scale_and_rotation_deg(sx, sy, rotation);
        self
    }

    #[inline(always)]
    /// rotation need to be in degree, will be converted into radians internally
    pub fn set_scale_and_rotation_deg(&mut self, sx: f32, sy: f32, rotation: f32) {
        let rad = rotation.to_radians();
        self.set_scale_and_rotation_rad(sx, sy, rad);
    }

    #[inline(always)]
    /// value must be a degree, will be converted into radians internally
    pub fn with_rotation_deg(mut self, deg: f32) -> Self {
        self.set_rotation_deg(deg);
        self
    }

    #[inline(always)]
    /// value must be a degree, will be converted into radians internally
    pub fn set_rotation_deg(&mut self, deg: f32) {
        let rad = deg.to_radians();
        let (sx, sy) = self.get_scale();
        self.set_scale_and_rotation_rad(sx, sy, rad);
    }

    #[inline(always)]
    /// value must be in radian
    pub fn with_rotation_rad(mut self, rad: f32) -> Self {
        self.set_rotation_rad(rad);
        self
    }

    #[inline(always)]
    /// value must be in radian
    pub fn set_rotation_rad(&mut self, rad: f32) {
        let (sx, sy) = self.get_scale();
        self.set_scale_and_rotation_rad(sx, sy, rad);
    }

    fn transpose(self) -> Matrix<N, M> {
        let mut ret: Matrix<N, M> = Matrix::zero();
        for m in 0..M {
            for n in 0..N { ret[n][m] = self[m][n] }
        }
        ret
    }

    /// matMxN * vectorM -> vectorN
    /// the matrix is transposed internally
    pub fn dot_vec(self, rhs: Vector<M, f32>) -> Vector<N, f32> {
        let mut ret = Vector::default();
        let t = self.transpose();
        for n in 0..N { ret[n] = t[n].dot(rhs) }
        ret
    }

    /// mat3x2  * mat4x3  -> mat4x2
    /// matMxN1 * matN2xM -> matN2xN1
    /// the matrix is transposed internally
    pub fn dot_mat<const N2: usize>(self, rhs: Matrix<N2, M>) -> Matrix<N2, N> {
        let mut ret: Matrix<N2, N> = Matrix::zero();
        for n in 0..N {
            ret[n] = self.dot_vec(rhs[n])
        }
        ret
    }
}

pub type Matrix4x4 = Matrix<4, 4>;

impl Matrix4x4 {
    pub const IDENTITY: Self = Self {
        inner: [
            Vec4f::new(1.0, 0.0, 0.0, 0.0),
            Vec4f::new(0.0, 1.0, 0.0, 0.0),
            Vec4f::new(0.0, 0.0, 1.0, 0.0),
            Vec4f::new(0.0, 0.0, 0.0, 1.0),
        ]
    };
}

pub type Matrix3x2 = Matrix<3, 2>;

impl Matrix3x2 {
    pub const IDENTITY: Self = Self {
        inner: [
            Vec2f::new(1.0, 0.0),
            Vec2f::new(0.0, 1.0),
            Vec2f::new(0.0, 0.0),
        ]
    };

    /// Internally use [`dot_vec()`](Self::dot_vec) method,
    /// but better as this one use less data
    pub fn transform_point(&self, point: Vec2f) -> Vec2f {
        Matrix2x2 { inner: [self[0], self[1]] }.dot_vec(point) + self[2]
    }
}

pub type Matrix2x2 = Matrix<2, 2>;

impl Matrix2x2 {
    pub const IDENTITY: Self = Self {
        inner: [
            Vec2f::new(1.0, 0.0),
            Vec2f::new(0.0, 1.0),
        ]
    };
}

#[cfg(test)]
mod matrix_test {
    use super::*;
    use crate::vector::Vec3f;

    #[test]
    fn mat_vec() {
        let mat3x2 = Matrix3x2::IDENTITY;
        let point3 = Vec3f::new(1.0, 1.0, 1.0);
        let point2 = Vec2f::new(1.0, 1.0);

        let res = mat3x2.dot_vec(point3);
        let cpr = mat3x2.transform_point(point2);

        eprintln!("{mat3x2:?}");
        assert_eq!(res, cpr);
    }

    #[test]
    fn mat_mul() {
        let mat2x2 = Matrix2x2::IDENTITY.with_rotation_rad(0.0);
        let mat3x2 = Matrix3x2::IDENTITY.with_translate(1.0, 1.0);

        let res = mat2x2.dot_mat(mat3x2);
        let cpr = Matrix3x2::IDENTITY;

        assert_eq!(res, cpr);
    }

    #[test]
    fn rotation() {
        let point = Vec2f::new(1.0, 1.2);
        let mat2x2 = Matrix2x2::IDENTITY.with_scale(1.3, 1.4).with_rotation_rad(30.);
        let res = mat2x2.dot_vec(point);

        let mat3x2 = Matrix3x2::IDENTITY.with_scale_and_rotation_rad(1.3, 1.4, 30.);
        let cpr = mat3x2.transform_point(point);

        assert_eq!(res, cpr);
    }
}
