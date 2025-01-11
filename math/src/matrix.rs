use std::ops::{Index, IndexMut};

use crate::Vector3;

// matrix 2x3
// [      x    y
//     ----------- 
//     [  1,  20 ],
//     [  9,   5 ],
//     [-13,  -6 ],
// ]
//
// drawn as
// x |  1   9  -13 |
// y | 20   5   -6 |

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Matrix<Vector, const N: usize> {
    data: [Vector; N]
}

impl std::fmt::Debug for Matrix<Vector3<f32>, 3> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = Vector3 { x: self[0].x, y: self[1].x, z: self[2].x };
        let y = Vector3 { x: self[0].y, y: self[1].y, z: self[2].y };
        let z = Vector3 { x: self[0].z, y: self[1].z, z: self[2].z };
        let conv = Self { data: [x, y, z] };
        conv.data.iter().enumerate().try_for_each(|(idx, vec3)| {
            let (prefix, suffix) = match idx {
                0 => ("x", "\n"),
                1 => ("y", "\n"),
                2 => ("z", ""),
                _ => unreachable!()
            };
            write!(f, "{prefix} | {:0.3} {:0.3} {:0.3} |{suffix}", vec3.x, vec3.y, vec3.z)
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

impl Matrix<Vector3<f32>, 3> {
    pub const IDENTITIY: Self = Self {
        data: [
            Vector3 { x: 1.0, y: 0.0, z: 0.0 }, // | x |
            Vector3 { x: 0.0, y: 1.0, z: 0.0 }, // | y |
            Vector3 { x: 0.0, y: 0.0, z: 1.0 }, // | z |
        ]
    };

    pub fn translation(&mut self, tx: f32, ty: f32) {
        self[2].x += tx;
        self[2].y += ty;
    }

    pub fn scale(&mut self, scale_x: f32, scale_y: f32) {
        self[0].x = scale_x;
        self[1].y = scale_y;
    }
}

// dot product
impl std::ops::Mul<Vector3<f32>> for Matrix<Vector3<f32>, 3> {
    type Output = Vector3<f32>;
    fn mul(self, rhs: Vector3<f32>) -> Self::Output {
        let x = Vector3 { x: self[0].x, y: self[1].x, z: self[2].x } * rhs;
        let y = Vector3 { x: self[0].y, y: self[1].y, z: self[2].y } * rhs;
        let z = Vector3 { x: self[0].z, y: self[1].z, z: self[2].z } * rhs;

        Vector3 { x, y, z }
    }
}
