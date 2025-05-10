use std::ops::{Index, IndexMut};

use crate::vector::Vector;

use super::{Vector4, GpuPrimitive, NumDebugger};

/// In GPU Matrix 2x3 means 3 `Vector<2, T>` stacked in horizontal order.
/// Matrix 4x2 means 2 rows of `[x, y, z, w]`.
/// So, [`Matrix<M, N, T>`] in CPU should be constructed by \[[`Vector<M, T>`]; `N`\].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Matrix<const M: usize, const N: usize, T: GpuPrimitive> {
    inner: [Vector<M, T>; N]
}

impl<const M: usize, const N: usize, T: GpuPrimitive + NumDebugger> std::fmt::Debug for Matrix<M, N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for m in 0..M {
            let (prefix, suffix) = match m {
                0 => ("x |", "|\n"),
                1 => ("y |", "|\n"),
                2 => ("z |", "|\n"),
                3 => ("w |", "|"),
                _ => unreachable!()
            };
            s.push_str(prefix);
            for n in 0..N {
                let num = self[n][m];
                if num.is_signed() {
                    s.push_str(format!("{num:0.3} ").as_str());
                } else {
                    s.push_str(format!(" {num:0.3} ").as_str());
                }
            }
            s.push_str(suffix);
        }
        write!(f, "{}", s)
    }
}

impl<const M: usize, const N: usize, T: GpuPrimitive> Index<usize> for Matrix<M, N, T> {
    type Output = Vector<M, T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<const M: usize, const N: usize, T: GpuPrimitive> IndexMut<usize> for Matrix<M, N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<const M: usize, const N: usize, T: GpuPrimitive> Matrix<M, N, T> {
    fn zero() -> Self {
        Self { inner: [ Vector::default(); N ] }
    }

    pub fn with_translate(mut self, x: T, y: T) -> Self {
        self[N - 1].set_x(x);
        self[N - 1].set_y(y);
        self
    }

    pub fn set_translate(&mut self, x: T, y: T) {
        self[N - 1].set_x(x);
        self[N - 1].set_y(y);
    }

    pub fn with_scale(mut self, w: T, h: T) -> Self {
        self[0].set_x(w);
        self[1].set_y(h);
        self
    }

    pub fn set_scale(&mut self, w: T, h: T) {
        self[0].set_x(w);
        self[1].set_y(h);
    }

    pub fn data(&self) -> &[Vector<M, T>] {
        &self.inner
    }

    fn transpose(self) -> Matrix<N, M, T> {
        let mut ret: Matrix<N, M, T> = Matrix::zero();
        for n in 0..N {
            for m in 0..M { ret[m][n] = self[n][m] }
        }
        ret
    }

    pub fn dot_vec(self, rhs: Vector<N, T>) -> Vector<N, T> {
        let mut ret = Vector::default();
        let t = self.transpose();
        for n in 0..N { ret[n] = t[n].dot(rhs) }
        ret
    }

    /// [`Matrix`] 2x3 => \[[`Vector<2, T>`]; 3\].
    /// in GPU this becomes:
    /// 
    /// x | v1x, v2x, v2x |
    /// y | v1y, v2y, v2y |
    /// 
    /// So the rule is: [`Matrix<M, N, T>`] * \[[`Vector<N, T>`]; Rows\]
    pub fn dot_mat<const M2: usize>(self, rhs: Matrix<N, M2, T>) -> Matrix<N, M2, T> {
        let mut ret: Matrix<N, M2, T> = Matrix::zero();
        for m in 0..M {
            ret[m] = self.dot_vec(rhs[m])
        }
        ret
    }
}

pub type Matrix4x4 = Matrix<4, 4, f32>;

impl Matrix<4, 4, f32> {
    pub const IDENTITY: Self = Self {
        inner: [
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ]
    };

    // pub fn transform(&mut self, tx: f32, ty: f32, sw: f32, sh: f32) {
    //     self[0].x = sw;
    //     self[1].y = sh;
    //     self[3].x = tx;
    //     self[3].y = ty;
    // }
}

// #[cfg(test)]
// mod matrix_test {
//     use super::*;

//     #[test]
//     fn matrix_test() {
//         let mut i = Matrix::IDENTITY;
//         i.translate(1., 2.);
//         let vec4f = Vec4f32::new(0., 1., 2., 3.);
//         let r1 = i.dot_vec(vec4f);
//         assert_eq!(Vector4::new(3., 7., 2., 3.), r1);

//         let mat4x4 = Matrix::<Vector4<f32>, 4>::IDENTITY;
//         let r2 = mat4x4.dot_mat(i);
//         assert_eq!(r2, Matrix {
//             data: [
//                 Vector4::new(1., 0., 0., 0.),
//                 Vector4::new(0., 1., 0., 0.),
//                 Vector4::new(0., 0., 1., 0.),
//                 Vector4::new(1., 2., 0., 1.),
//             ]
//         });
//     }
// }
