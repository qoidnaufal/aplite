use super::{GpuPrimitive, NumDebugger};

#[derive(Clone, Copy)]
pub struct Vector<const N: usize, T: GpuPrimitive> {
    pub(crate) inner: [T; N]
}

impl<const N: usize, T: GpuPrimitive> Default for Vector<N, T> {
    fn default() -> Self {
        Self { inner: [T::default(); N] }
    }
}

impl<const N: usize, T: GpuPrimitive> Vector<N, T> {
    #[inline(always)]
    pub(crate) const fn new_from_array(inner: [T; N]) -> Self { Self { inner } }

    #[inline(always)]
    pub const fn as_slice(&self) -> &[T] { &self.inner }

    #[inline(always)]
    pub const fn as_slice_mut(&mut self) -> &mut [T] { &mut self.inner }

    #[inline(always)]
    pub const fn into_array(self) -> [T; N] { self.inner }

    pub fn dot(self, rhs: Self) -> T {
        let mut ret = T::default();
        for n in 0..N { ret += self[n] * rhs[n] }
        ret
    }

    pub fn cross(self, rhs: Self) -> Self {
        let mut ret = self;
        for n in 0..N { ret.inner[n] *= rhs.inner[n] }
        ret
    }
}
pub type Vec2f = Vector<2, f32>;

impl Vec2f {
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self { Self { inner: [x, y] } }

    #[inline(always)]
    pub const fn x(&self) -> f32 { self.inner[0] }

    #[inline(always)]
    pub const fn set_x(&mut self, x: f32) { self.inner[0] = x }

    #[inline(always)]
    pub const fn add_x(&mut self, x: f32) { self.inner[0] += x }

    #[inline(always)]
    pub const fn sub_x(&mut self, x: f32) { self.inner[0] -= x }

    #[inline(always)]
    pub const fn mul_x(&mut self, x: f32) { self.inner[0] *= x }

    #[inline(always)]
    pub const fn div_x(&mut self, x: f32) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> f32 { self.inner[1] }

    #[inline(always)]
    pub const fn set_y(&mut self, y: f32) { self.inner[1] = y }

    #[inline(always)]
    pub const fn add_y(&mut self, y: f32) { self.inner[1] += y }

    #[inline(always)]
    pub const fn sub_y(&mut self, y: f32) { self.inner[1] -= y }

    #[inline(always)]
    pub const fn mul_y(&mut self, y: f32) { self.inner[1] *= y }

    #[inline(always)]
    pub const fn div_y(&mut self, y: f32) { self.inner[1] /= y }
}

pub type Vec2u = Vector<2, u32>;

impl Vec2u {
    #[inline(always)]
    pub const fn new(x: u32, y: u32) -> Self { Self { inner: [x, y] } }

    #[inline(always)]
    pub const fn x(&self) -> u32 { self.inner[0] }

    #[inline(always)]
    pub const fn set_x(&mut self, x: u32) { self.inner[0] = x }

    #[inline(always)]
    pub const fn add_x(&mut self, x: u32) { self.inner[0] += x }

    #[inline(always)]
    pub const fn sub_x(&mut self, x: u32) { self.inner[0] -= x }

    #[inline(always)]
    pub const fn mul_x(&mut self, x: u32) { self.inner[0] *= x }

    #[inline(always)]
    pub const fn div_x(&mut self, x: u32) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> u32 { self.inner[1] }

    #[inline(always)]
    pub const fn set_y(&mut self, y: u32) { self.inner[1] = y }

    #[inline(always)]
    pub const fn add_y(&mut self, y: u32) { self.inner[1] += y }

    #[inline(always)]
    pub const fn sub_y(&mut self, y: u32) { self.inner[1] -= y }

    #[inline(always)]
    pub const fn mul_y(&mut self, y: u32) { self.inner[1] *= y }

    #[inline(always)]
    pub const fn div_y(&mut self, y: u32) { self.inner[1] /= y }
}

pub type Vec3f = Vector<3, f32>;

impl Vec3f {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { inner: [x, y, z] } }

    #[inline(always)]
    pub const fn x(&self) -> f32 { self.inner[0] }

    #[inline(always)]
    pub const fn set_x(&mut self, x: f32) { self.inner[0] = x }

    #[inline(always)]
    pub const fn add_x(&mut self, x: f32) { self.inner[0] += x }

    #[inline(always)]
    pub const fn sub_x(&mut self, x: f32) { self.inner[0] -= x }

    #[inline(always)]
    pub const fn mul_x(&mut self, x: f32) { self.inner[0] *= x }

    #[inline(always)]
    pub const fn div_x(&mut self, x: f32) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> f32 { self.inner[1] }

    #[inline(always)]
    pub const fn set_y(&mut self, y: f32) { self.inner[1] = y }

    #[inline(always)]
    pub const fn add_y(&mut self, y: f32) { self.inner[1] += y }

    #[inline(always)]
    pub const fn sub_y(&mut self, y: f32) { self.inner[1] -= y }

    #[inline(always)]
    pub const fn mul_y(&mut self, y: f32) { self.inner[1] *= y }

    #[inline(always)]
    pub const fn div_y(&mut self, y: f32) { self.inner[1] /= y }

    #[inline(always)]
    pub const fn z(&self) -> f32 { self.inner[2] }

    #[inline(always)]
    pub const fn set_z(&mut self, z: f32) { self.inner[2] = z }

    #[inline(always)]
    pub const fn add_z(&mut self, z: f32) { self.inner[2] += z }

    #[inline(always)]
    pub const fn sub_z(&mut self, z: f32) { self.inner[2] -= z }

    #[inline(always)]
    pub const fn mul_z(&mut self, z: f32) { self.inner[2] *= z }

    #[inline(always)]
    pub const fn div_z(&mut self, z: f32) { self.inner[2] /= z }
}

pub type Vec3u = Vector<3, u32>;

impl Vec3u {
    #[inline(always)]
    pub const fn new(x: u32, y: u32, z: u32) -> Self { Self { inner: [x, y, z] } }

    #[inline(always)]
    pub const fn x(&self) -> u32 { self.inner[0] }

    #[inline(always)]
    pub const fn set_x(&mut self, x: u32) { self.inner[0] = x }

    #[inline(always)]
    pub const fn add_x(&mut self, x: u32) { self.inner[0] += x }

    #[inline(always)]
    pub const fn sub_x(&mut self, x: u32) { self.inner[0] -= x }

    #[inline(always)]
    pub const fn mul_x(&mut self, x: u32) { self.inner[0] *= x }

    #[inline(always)]
    pub const fn div_x(&mut self, x: u32) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> u32 { self.inner[1] }

    #[inline(always)]
    pub const fn set_y(&mut self, y: u32) { self.inner[1] = y }

    #[inline(always)]
    pub const fn add_y(&mut self, y: u32) { self.inner[1] += y }

    #[inline(always)]
    pub const fn sub_y(&mut self, y: u32) { self.inner[1] -= y }

    #[inline(always)]
    pub const fn mul_y(&mut self, y: u32) { self.inner[1] *= y }

    #[inline(always)]
    pub const fn div_y(&mut self, y: u32) { self.inner[1] /= y }

    #[inline(always)]
    pub const fn z(&self) -> u32 { self.inner[2] }

    #[inline(always)]
    pub const fn set_z(&mut self, z: u32) { self.inner[2] = z }

    #[inline(always)]
    pub const fn add_z(&mut self, z: u32) { self.inner[2] += z }

    #[inline(always)]
    pub const fn sub_z(&mut self, z: u32) { self.inner[2] -= z }

    #[inline(always)]
    pub const fn mul_z(&mut self, z: u32) { self.inner[2] *= z }

    #[inline(always)]
    pub const fn div_z(&mut self, z: u32) { self.inner[2] /= z }
}

pub type Vec4f = Vector<4, f32>;

impl Vec4f {
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { inner: [x, y, z, w] } }

    #[inline(always)]
    pub const fn x(&self) -> f32 { self.inner[0] }

    #[inline(always)]
    pub const fn set_x(&mut self, x: f32) { self.inner[0] = x }

    #[inline(always)]
    pub const fn add_x(&mut self, x: f32) { self.inner[0] += x }

    #[inline(always)]
    pub const fn sub_x(&mut self, x: f32) { self.inner[0] -= x }

    #[inline(always)]
    pub const fn mul_x(&mut self, x: f32) { self.inner[0] *= x }

    #[inline(always)]
    pub const fn div_x(&mut self, x: f32) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> f32 { self.inner[1] }

    #[inline(always)]
    pub const fn set_y(&mut self, y: f32) { self.inner[1] = y }

    #[inline(always)]
    pub const fn add_y(&mut self, y: f32) { self.inner[1] += y }

    #[inline(always)]
    pub const fn sub_y(&mut self, y: f32) { self.inner[1] -= y }

    #[inline(always)]
    pub const fn mul_y(&mut self, y: f32) { self.inner[1] *= y }

    #[inline(always)]
    pub const fn div_y(&mut self, y: f32) { self.inner[1] /= y }

    #[inline(always)]
    pub const fn z(&self) -> f32 { self.inner[2] }

    #[inline(always)]
    pub const fn set_z(&mut self, z: f32) { self.inner[2] = z }

    #[inline(always)]
    pub const fn add_z(&mut self, z: f32) { self.inner[2] += z }

    #[inline(always)]
    pub const fn sub_z(&mut self, z: f32) { self.inner[2] -= z }

    #[inline(always)]
    pub const fn mul_z(&mut self, z: f32) { self.inner[2] *= z }

    #[inline(always)]
    pub const fn div_z(&mut self, z: f32) { self.inner[2] /= z }

    #[inline(always)]
    pub const fn w(&self) -> f32 { self.inner[3] }

    #[inline(always)]
    pub const fn set_w(&mut self, w: f32) { self.inner[3] = w }

    #[inline(always)]
    pub const fn add_w(&mut self, w: f32) { self.inner[3] += w }

    #[inline(always)]
    pub const fn sub_w(&mut self, w: f32) { self.inner[3] -= w }

    #[inline(always)]
    pub const fn mul_w(&mut self, w: f32) { self.inner[3] *= w }

    #[inline(always)]
    pub const fn div_w(&mut self, w: f32) { self.inner[3] /= w }
}

pub type Vec4u = Vector<4, u32>;

impl Vec4u {
    #[inline(always)]
    pub const fn new(x: u32, y: u32, z: u32, w: u32) -> Self { Self { inner: [x, y, z, w] } }

    #[inline(always)]
    pub const fn x(&self) -> u32 { self.inner[0] }

    #[inline(always)]
    pub const fn set_x(&mut self, x: u32) { self.inner[0] = x }

    #[inline(always)]
    pub const fn add_x(&mut self, x: u32) { self.inner[0] += x }

    #[inline(always)]
    pub const fn sub_x(&mut self, x: u32) { self.inner[0] -= x }

    #[inline(always)]
    pub const fn mul_x(&mut self, x: u32) { self.inner[0] *= x }

    #[inline(always)]
    pub const fn div_x(&mut self, x: u32) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> u32 { self.inner[1] }

    #[inline(always)]
    pub const fn set_y(&mut self, y: u32) { self.inner[1] = y }

    #[inline(always)]
    pub const fn add_y(&mut self, y: u32) { self.inner[1] += y }

    #[inline(always)]
    pub const fn sub_y(&mut self, y: u32) { self.inner[1] -= y }

    #[inline(always)]
    pub const fn mul_y(&mut self, y: u32) { self.inner[1] *= y }

    #[inline(always)]
    pub const fn div_y(&mut self, y: u32) { self.inner[1] /= y }

    #[inline(always)]
    pub const fn z(&self) -> u32 { self.inner[2] }

    #[inline(always)]
    pub const fn set_z(&mut self, z: u32) { self.inner[2] = z }

    #[inline(always)]
    pub const fn add_z(&mut self, z: u32) { self.inner[2] += z }

    #[inline(always)]
    pub const fn sub_z(&mut self, z: u32) { self.inner[2] -= z }

    #[inline(always)]
    pub const fn mul_z(&mut self, z: u32) { self.inner[2] *= z }

    #[inline(always)]
    pub const fn div_z(&mut self, z: u32) { self.inner[2] /= z }

    #[inline(always)]
    pub const fn w(&self) -> u32 { self.inner[3] }

    #[inline(always)]
    pub const fn set_w(&mut self, w: u32) { self.inner[3] = w }

    #[inline(always)]
    pub const fn add_w(&mut self, w: u32) { self.inner[3] += w }

    #[inline(always)]
    pub const fn sub_w(&mut self, w: u32) { self.inner[3] -= w }

    #[inline(always)]
    pub const fn mul_w(&mut self, w: u32) { self.inner[3] *= w }

    #[inline(always)]
    pub const fn div_w(&mut self, w: u32) { self.inner[3] /= w }
}

// arithmethic operations

impl<const N: usize, T: GpuPrimitive> std::ops::Add<Self> for Vector<N, T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = self;
        for i in 0..N { ret.inner[i] += rhs.inner[i] }
        ret
    }
}

impl<const N: usize, T: GpuPrimitive> std::ops::Sub<Self> for Vector<N, T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = self;
        for i in 0..N { ret.inner[i] -= rhs.inner[i] }
        ret
    }
}

impl<const N: usize, T: GpuPrimitive> std::ops::Mul<T> for Vector<N, T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let mut ret = self;
        for i in 0..N { ret.inner[i] *= rhs }
        ret
    }
}

impl<const N: usize, T: GpuPrimitive> std::ops::Div<T> for Vector<N, T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let mut ret = self;
        for i in 0..N { ret.inner[i] /= rhs }
        ret
    }
}

// logical operations

impl<const N: usize, T: GpuPrimitive> PartialEq for Vector<N, T> {
    fn eq(&self, other: &Self) -> bool {
        let mut ret = false;
        for i in 0..N { ret = self.inner[i] == other.inner[i] }
        ret
    }
}

impl<const N: usize, T: GpuPrimitive> Eq for Vector<N, T> {}

// type conversion

impl<const N: usize> Vector<N, f32> {
    pub fn u32(self) -> Vector<N, u32> {
        self.into()
    }
}

impl<const N: usize> Vector<N, u32> {
    pub fn f32(self) -> Vector<N, f32> {
        self.into()
    }
}

impl<const N: usize> From<Vector<N, f32>> for Vector<N, u32> {
    fn from(value: Vector<N, f32>) -> Self {
        let mut ret = Self::new_from_array([0; N]);
        for i in 0..N { ret.inner[i] = value.inner[i].round() as _ }
        ret
    }
}

impl<const N: usize> From<Vector<N, u32>> for Vector<N, f32> {
    fn from(value: Vector<N, u32>) -> Self {
        let mut ret = Self::new_from_array([0.0; N]);
        for i in 0..N { ret.inner[i] = value.inner[i] as _ }
        ret
    }
}

impl<T: GpuPrimitive> From<(T, T)> for Vector<2, T> {
    fn from(value: (T, T)) -> Self {
        Self::new_from_array([value.0, value.1])
    }
}

// other operations

impl<const N: usize, T: NumDebugger> std::fmt::Debug for Vector<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("Vector{N}");
        let s = self.debug_formatter(name.as_str());
        write!(f, "{s}")
    }
}

impl<const N: usize, T: GpuPrimitive> std::ops::Index<usize> for Vector<N, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<const N: usize, T: GpuPrimitive> std::ops::IndexMut<usize> for Vector<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<const N: usize, T: NumDebugger> Vector<N, T> {
    pub(crate) fn debug_formatter(&self, name: &str) -> String {
        let mut s = String::new();
        s.push_str(format!("{name} {{").as_str());
        for n in 0..N {
            let num = self[n];
            let num_str = if num.is_signed() {
                format!(" {num:0.3}")
            } else if num.is_float() {
                format!("  {num:0.3}")
            } else {
                format!(" {num}")
            };
            if n == N - 1 {
                if num.is_float() {
                    s.push_str(format!("{num_str}  }}").as_str());
                } else {
                    s.push_str(format!("{num_str} }}").as_str());
                }
            } else {
                s.push_str(format!("{num_str},").as_str());
            }
        }
        s
    }
}
