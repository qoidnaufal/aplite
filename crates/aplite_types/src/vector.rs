use super::{GpuPrimitive, NumDebugger};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector<const N: usize, T: GpuPrimitive> {
    inner: [T; N]
}

impl<const N: usize, T: NumDebugger> std::fmt::Debug for Vector<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("Vector{N}");
        let s = self.debug_formatter(name.as_str());
        write!(f, "{s}")
    }
}

impl<const N: usize, T: GpuPrimitive> Default for Vector<N, T> {
    fn default() -> Self {
        Self { inner: [T::default(); N] }
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

impl<const N: usize, T: GpuPrimitive> Vector<N, T> {
    pub const fn slice(self) -> [T; N] { self.inner }

    #[inline(always)]
    pub const fn x(&self) -> T { self.inner[0] }

    pub fn set_x(&mut self, x: T) { self.inner[0] = x }
    pub fn add_x(&mut self, x: T) { self.inner[0] += x }
    pub fn sub_x(&mut self, x: T) { self.inner[0] -= x }
    pub fn mul_x(&mut self, x: T) { self.inner[0] *= x }
    pub fn div_x(&mut self, x: T) { self.inner[0] /= x }

    #[inline(always)]
    pub const fn y(&self) -> T { self.inner[1] }

    pub fn set_y(&mut self, y: T) { self.inner[1] = y }
    pub fn add_y(&mut self, y: T) { self.inner[1] += y }
    pub fn sub_y(&mut self, y: T) { self.inner[1] -= y }
    pub fn mul_y(&mut self, y: T) { self.inner[1] *= y }
    pub fn div_y(&mut self, y: T) { self.inner[1] /= y }

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

// ... vec2

pub type Vector2<T> = Vector<2, T>;

impl<T: GpuPrimitive> Vector2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Vector { inner: [x, y] }
    }
}

// ... vec3

pub type Vector3<T> = Vector<3, T>;

impl<T: GpuPrimitive> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Vector { inner: [x, y, z ] }
    }
}

impl<T: GpuPrimitive> Vector3<T> {
    #[inline(always)]
    pub const fn z(&self) -> T { self.inner[2] }

    pub fn set_z(&mut self, z: T) { self.inner[2] = z }
    pub fn add_z(&mut self, z: T) { self.inner[2] += z }
    pub fn sub_z(&mut self, z: T) { self.inner[2] -= z }
    pub fn mul_z(&mut self, z: T) { self.inner[2] *= z }
    pub fn div_z(&mut self, z: T) { self.inner[2] /= z }
}

// ... vec4

pub type Vector4<T> = Vector<4, T>;

impl<T: GpuPrimitive> Vector4<T> {
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Vector { inner: [x, y, z, w] }
    }
}

impl<T: GpuPrimitive> Vector4<T> {
    #[inline(always)]
    pub const fn z(&self) -> T { self.inner[2] }

    pub fn set_z(&mut self, z: T) { self.inner[2] = z }
    pub fn add_z(&mut self, z: T) { self.inner[2] += z }
    pub fn sub_z(&mut self, z: T) { self.inner[2] -= z }
    pub fn mul_z(&mut self, z: T) { self.inner[2] *= z }
    pub fn div_z(&mut self, z: T) { self.inner[2] /= z }

    #[inline(always)]
    pub const fn w(&self) -> T { self.inner[3] }

    pub fn set_w(&mut self, w: T) { self.inner[3] = w }
    pub fn add_w(&mut self, w: T) { self.inner[3] += w }
    pub fn sub_w(&mut self, w: T) { self.inner[3] -= w }
    pub fn mul_w(&mut self, w: T) { self.inner[3] *= w }
    pub fn div_w(&mut self, w: T) { self.inner[3] /= w }
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
        let mut ret = Self::default();
        for i in 0..N { ret.inner[i] = value.inner[i].round() as _ }
        ret
    }
}

impl<const N: usize> From<Vector<N, u32>> for Vector<N, f32> {
    fn from(value: Vector<N, u32>) -> Self {
        let mut ret = Self::default();
        for i in 0..N { ret.inner[i] = value.inner[i] as _ }
        ret
    }
}

impl<T: GpuPrimitive> From<(T, T)> for Vector2<T> {
    fn from(value: (T, T)) -> Self {
        Self::new(value.0, value.1)
    }
}
