use std::ops::{Index, IndexMut};

use crate::{Vector2, Vector4};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Matrix<Vector, const N: usize> {
    data: [Vector; N]
}

impl std::fmt::Debug for Matrix<Vector4<f32>, 4> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mat = self.transpose();
        mat.data.iter().enumerate().try_for_each(|(idx, vec4)| {
            let prefix = match idx {
                0 => "\nx",
                1 => "\ny",
                2 => "\nz",
                3 => "\nw",
                _ => unreachable!()
            };
            write!(
                f,
                "{prefix} | {:0.3} {:0.3} {:0.3} {:0.3} |",
                vec4.x, vec4.y, vec4.z, vec4.w
            )
        })
    }
}

impl<Vector, const N: usize> Index<usize> for Matrix<Vector, N> {
    type Output = Vector;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<Vector, const N: usize> IndexMut<usize> for Matrix<Vector, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Matrix<Vector2<f32>, 2> {
    pub fn rotate(r: f32) -> Self {
        Self {
            data: [
                Vector2 { x: r.cos(), y: -r.sin() },
                Vector2 { x: r.sin(), y: r.cos() },
            ],
        }
    }

    pub fn dot_vec(self, vec2f: Vector2<f32>) -> Vector2<f32> {
        Vector2 {
            x: self[0].dot(vec2f),
            y: self[1].dot(vec2f),
        }
    }
}

impl Matrix<Vector4<f32>, 4> {
    pub const IDENTITY: Self = Self {
        data: [
            Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
        ]
    };

    fn transpose(self) -> Self {
        let x = Vector4 { x: self[0].x, y: self[1].x, z: self[2].x, w: self[3].x };
        let y = Vector4 { x: self[0].y, y: self[1].y, z: self[2].y, w: self[3].y };
        let z = Vector4 { x: self[0].z, y: self[1].z, z: self[2].z, w: self[3].z };
        let w = Vector4 { x: self[0].w, y: self[1].w, z: self[2].w, w: self[3].w };
        Self { data: [x, y, z, w] }
    }

    pub fn transform(&mut self, tx: f32, ty: f32, sw: f32, sh: f32) {
        self[0].x = sw;
        self[1].y = sh;
        self[3].x = tx;
        self[3].y = ty;
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        self[3].x = tx;
        self[3].y = ty;
    }

    pub fn scale(&mut self, sw: f32, sh: f32) {
        self[0].x = sw;
        self[1].y = sh;
    }

    pub fn data(&self) -> &[Vector4<f32>] {
        &self.data
    }

    pub fn dot_vec(self, vec4: Vector4<f32>) -> Vector4<f32> {
        let t = self.transpose();
        Vector4 {
            x: t[0].dot(vec4),
            y: t[1].dot(vec4),
            z: t[2].dot(vec4),
            w: t[3].dot(vec4),
        }
    }

    pub fn dot_mat(self, mat4x4: Self) -> Self {
        Self {
            data: [
                self.dot_vec(mat4x4[0]),
                self.dot_vec(mat4x4[1]),
                self.dot_vec(mat4x4[2]),
                self.dot_vec(mat4x4[3]),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_test() {
        let mut i = Matrix::<Vector4<f32>, 4>::IDENTITY;
        i.translate(1., 2.);
        let vec4f = Vector4::<f32>::new(0., 1., 2., 3.);
        let r1 = i.dot_vec(vec4f);
        assert_eq!(Vector4::new(3., 7., 2., 3.), r1);

        let mat4x4 = Matrix::<Vector4<f32>, 4>::IDENTITY;
        let r2 = mat4x4.dot_mat(i);
        assert_eq!(r2, Matrix {
            data: [
                Vector4::new(1., 0., 0., 0.),
                Vector4::new(0., 1., 0., 0.),
                Vector4::new(0., 0., 1., 0.),
                Vector4::new(1., 2., 0., 1.),
            ]
        });
    }
}
