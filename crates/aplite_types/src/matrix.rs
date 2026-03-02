use crate::vector::{Vec2u, Vec2f};

/// mat3x2 is composed as \[\[f32; 2\]; 3\]
/// # Representation:
/// Composition:   │ Multiplication:
/// (x , y)        │ x: x0, x1, x2
/// x0 , y0        │ y: y0, y1, y2
/// x1 , y1        │ 
/// x2 , y2        │ 
#[derive(Clone, Copy)]
pub struct Matrix3x2([f32; 6]);

impl Matrix3x2 {
    pub const IDENTITY: Self = Self([
        1.0, 0.0,
        0.0, 1.0,
        0.0, 0.0,
    ]);

    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    pub const fn as_array(&self) -> [f32; 6] {
        self.0
    }

    pub const fn scale(&self) -> [f32; 2] {
        [self.0[0], self.0[3]]
    }

    pub const fn translate(&self) -> Vec2f {
        Vec2f {
            x: self.0[4],
            y: self.0[5],
        }
    }

    pub const fn x_axis(&self) -> Vec2f {
        Vec2f {
            x: self.0[0],
            y: self.0[2],
        }
    }

    pub const fn y_axis(&self) -> Vec2f {
        Vec2f {
            x: self.0[1],
            y: self.0[3],
        }
    }

    pub const fn from_scale(sx: f32, sy: f32) -> Self {
        Self([
            sx, 0.,
            0., sy,
            0., 0.,
        ])
    }

    pub fn from_scale_rotate_rad(sx: f32, sy: f32, rad: f32) -> Self {
        let (sin, cos) = rad.sin_cos();
        Self([
            sx * cos, sx * -sin,
            sy * sin, sy *  cos,
            0., 0.,
        ])
    }

    pub fn from_rotate_deg(deg: f32) -> Self {
        let rad = deg.to_radians();
        Self([
            rad.cos(), -rad.sin(),
            rad.sin(),  rad.cos(),
            0., 0.,
        ])
    }

    pub fn from_rotate_rad(rad: f32) -> Self {
        Self([
            rad.cos(), -rad.sin(),
            rad.sin(),  rad.cos(),
            0., 0.,
        ])
    }

    pub const fn from_translate(tx: f32, ty: f32) -> Self {
        Self([
            0., 0.,
            0., 0.,
            tx, ty,
        ])
    }

    pub const fn from_scale_translate(sx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        Self([
            sx, 0.,
            0., sy,
            tx, ty,
        ])
    }

    pub fn from_scale_deg_translate(sx: f32, sy: f32, deg: f32, tx: f32, ty: f32) -> Self {
        let rad = deg.to_radians();
        let (sin, cos) = rad.sin_cos();
        Self([
            sx * cos, sx * -sin,
            sy * sin, sy *  cos,
            tx, ty,
        ])
    }

    // FIXME: why did i get zero value on 90 degree
    pub fn from_scale_rad_translate(sx: f32, sy: f32, rad: f32, tx: f32, ty: f32) -> Self {
        let (sin, cos) = rad.sin_cos();
        Self([
            sx * cos, sx * -sin,
            sy * sin, sy *  cos,
            tx, ty,
        ])
    }

    pub fn with_scale(mut self, sx: f32, sy: f32) -> Self {
        self.set_scale(sx, sy);
        self
    }

    pub fn with_rotate_rad(mut self, rad: f32) -> Self {
        self.set_rotate_rad(rad);
        self
    }

    pub fn with_rotate_deg(mut self, deg: f32) -> Self {
        self.set_rotate_deg(deg);
        self
    }

    pub fn with_translate(mut self, tx: f32, ty: f32) -> Self {
        self.set_translate(tx, ty);
        self
    }

    pub fn set_scale(&mut self, sx: f32, sy: f32) {
        self[0] = sx;
        self[3] = sy;
    }

    pub fn adjust_scale(&mut self, sx: f32, sy: f32) {
        self[0] *= sx;
        self[3] *= sy;
    }

    pub fn set_rotate_rad(&mut self, rad: f32) {
        let (sin, cos) = rad.sin_cos();
        self[0] =  cos;
        self[1] = -sin;
        self[2] =  sin;
        self[3] =  cos;
    }

    pub fn set_rotate_deg(&mut self, deg: f32) {
        self.set_rotate_rad(deg.to_radians());
    }

    pub fn set_translate(&mut self, tx: f32, ty: f32) {
        self[4] = tx;
        self[5] = ty;
    }

    pub fn transform_point(&self, point: Vec2u) -> Vec2u {
        let p = point.vec2f();
        let vec2f = self.transform_vec2f(p);
        vec2f.vec2u()
    }

    pub fn transform_vec2f(&self, vec2f: Vec2f) -> Vec2f {
        let x = self.x_axis().dot(vec2f);
        let y = self.y_axis().dot(vec2f);
        Vec2f::new(x, y) + self.translate()
    }
}

impl PartialEq for Matrix3x2 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::ops::Index<usize> for Matrix3x2 {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Matrix3x2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl std::fmt::Debug for Matrix3x2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push('\n');
        for n in 0..2 {
            let (prefix, suffix) = match n {
                0 => ("x │", "│\n"),
                1 => ("y │", "│\n"),
                2 => ("z │", "│\n"),
                _ => panic!()
            };
            s.push_str(prefix);
            for m in 0..3 {
                let num = self[n + m * 2];
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

#[cfg(test)]
mod matrix_test {
    use crate::vector::{Vec2f, Vec2u};
    use super::Matrix3x2;

    #[test]
    fn mat_vec() {
        let mat3x2 = Matrix3x2::from_scale_translate(2.0, 3.0, 4.0, 5.0);
        let point = Vec2u::new(100, 100);

        let res = mat3x2.transform_point(point).vec2f();
        let cpr = Vec2f::new(204.0, 305.0);

        eprintln!("{mat3x2:?}");
        assert_eq!(res, cpr);
    }
}
