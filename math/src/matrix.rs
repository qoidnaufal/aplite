use std::ops::{Index, IndexMut};

use crate::Vector3;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl From<Vector3<f32>> for Vector4<f32> {
    fn from(v3: Vector3<f32>) -> Self {
        Self {
            x: v3.x,
            y: v3.y,
            z: v3.z,
            w: 1.0,
        }
    }
}

impl<T> std::ops::Mul<Self> for Vector4<T>
where T:
    std::ops::Add<T, Output = T>
    + std::ops::Mul<T, Output = T>
    + Copy
{
    type Output = T;
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Matrix<Vector, const N: usize> {
    data: [Vector; N]
}

impl Matrix<Vector4<f32>, 4> {
    fn convert(&self) -> [Vector4<f32>; 4] {
        let x = Vector4 { x: self[0].x, y: self[1].x, z: self[2].x, w: self[3].x };
        let y = Vector4 { x: self[0].y, y: self[1].y, z: self[2].y, w: self[3].y };
        let z = Vector4 { x: self[0].z, y: self[1].z, z: self[2].z, w: self[3].z };
        let w = Vector4 { x: self[0].w, y: self[1].w, z: self[2].w, w: self[3].w };
        [x, y, z, w]
    }
}

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

impl std::fmt::Debug for Matrix<Vector4<f32>, 4> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mat = self.convert();
        mat.iter().enumerate().try_for_each(|(idx, vec4)| {
            let (prefix, suffix) = match idx {
                0 => ("x", "\n"),
                1 => ("y", "\n"),
                2 => ("z", "\n"),
                3 => ("w",  "" ),
                _ => unreachable!()
            };
            write!(
                f,
                "{prefix} | {:0.3} {:0.3} {:0.3} {:0.3} |{suffix}",
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

// glam's implementation
// Vector3 | vx | -> Vector3x { x * vx, y * vx, z * vx }
// Vector3 | vy | -> Vector3y { x * vy, y * vy, z * vy }
// Vector3 | vz | -> Vector3z { x * vz, y * vz, z * vz }
//
// Vector3x + Vector3y + Vector3z
// Vector3 {
//     x: (x * vx) + (x * vy) + (x * vz),
//     y: (y * vx) + (y * vy) + (y * vz),
//     z: (z * vx) + (z * vy) + (z * vz),
// }
impl Matrix<Vector4<f32>, 4> {
    pub const IDENTITIY: Self = Self {
        data: [
            Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
            Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
        ]
    };

    pub fn transform(&mut self, tx: f32, ty: f32, sw: f32, sh: f32) {
        self[0].x = sw;
        self[1].y = sh;
        self[3].x = tx;
        self[3].y = ty;
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        self[3].x += tx;
        self[3].y += ty;
    }

    pub fn scale(&mut self, sw: f32, sh: f32) {
        self[0].x = sw;
        self[1].y = sh;
    }

    pub fn data(&self) -> &[Vector4<f32>] {
        &self.data
    }
}

impl std::ops::Mul<Vector4<f32>> for Matrix<Vector4<f32>, 4> {
    type Output = Vector4<f32>;
    fn mul(self, rhs: Vector4<f32>) -> Self::Output {
        let conv = self.convert();
        let x = conv[0] * rhs;
        let y = conv[1] * rhs;
        let z = conv[2] * rhs;
        let w = conv[3] * rhs;

        Vector4 { x, y, z, w }
    }
}

//   matrix A  *  matrix B
// [ x x x x ]  [ x y z w ] |
// [ y y y y ]  [ x y z w ] |
// [ z z z z ]  [ x y z w ] |
// [ w w w w ]  [ x y z w ] |

impl std::ops::Mul for Matrix<Vector4<f32>, 4> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let conv = self.convert();
        let rhs = rhs.convert();
        let x = Vector4 { x: conv[0] * rhs[0], y: conv[0] * rhs[1], z: conv[0] * rhs[2], w: conv[0] * rhs[3] };
        let y = Vector4 { x: conv[1] * rhs[0], y: conv[1] * rhs[1], z: conv[1] * rhs[2], w: conv[1] * rhs[3] };
        let z = Vector4 { x: conv[2] * rhs[0], y: conv[2] * rhs[1], z: conv[2] * rhs[2], w: conv[2] * rhs[3] };
        let w = Vector4 { x: conv[3] * rhs[0], y: conv[3] * rhs[1], z: conv[3] * rhs[2], w: conv[3] * rhs[3] };

        Self {
            data: [x, y, z, w],
        }
    }
}
