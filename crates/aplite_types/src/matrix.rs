use std::ops::{Index, IndexMut};

use crate::{vector::Vector, Vector2};

use super::{Vector4, GpuPrimitive, NumDebugger};

/// In GPU mat3x2 is actually a 3 [`Vector<2, T>`] in CPU
/// So, [`Matrix<N, M, T>`] should be constructed by a \[[`Vector<M, T>`]; `N`\].
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

    pub fn scale(&self) -> [T; 2] {
        [self.inner[0].x(), self.inner[1].y()]
    }

    #[inline(always)]
    pub fn with_translate(mut self, x: T, y: T) -> Self {
        self[N - 1].set_x(x);
        self[N - 1].set_y(y);
        self
    }

    #[inline(always)]
    pub fn set_translate(&mut self, x: T, y: T) {
        self[N - 1].set_x(x);
        self[N - 1].set_y(y);
    }

    #[inline(always)]
    pub fn with_scale(mut self, w: T, h: T) -> Self {
        self[0].set_x(w);
        self[1].set_y(h);
        self
    }

    #[inline(always)]
    pub fn set_scale(&mut self, w: T, h: T) {
        self[0].set_x(w);
        self[1].set_y(h);
    }

    #[inline(always)]
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

impl Matrix4x4 {
    pub const IDENTITY: Self = Self {
        inner: [
            Vector4::new(1.0, 0.0, 0.0, 0.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
        ]
    };
}

pub type Matrix3x2 = Matrix<2, 3, f32>;

impl Matrix3x2 {
    pub const IDENTITY: Self = Self {
        inner: [
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(0.0, 0.0),
        ]
    };
}
