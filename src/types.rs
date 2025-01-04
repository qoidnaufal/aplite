use crate::error::Error;

#[derive(Debug,Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T> From<winit::dpi::PhysicalSize<T>> for Size<T> {
    fn from(val: winit::dpi::PhysicalSize<T>) -> Self {
        Self {
            width: val.width,
            height: val.height,
        }
    }
}

impl PartialEq for Size<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl Eq for Size<u32> {}

#[derive(Debug, Clone, Copy)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T
}

// impl<T: Default + std::ops::Add + std::ops::AddAssign> Vector3<T> {
//     pub fn new() -> Self {
//         Self { x: T::default(), y: T::default(), z: T::default() }
//     }

//     pub fn translation(&mut self, t: Vector2<T>) {
//         self.x += t.x;
//         self.y += t.y;
//     }
// }

impl<T: Default> From<Vector2<T>> for Vector3<T> {
    fn from(val: Vector2<T>) -> Self {
        Self {
            x: val.x,
            y: val.y,
            z: T::default()
        }
    }
}

impl PartialEq for Vector3<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.z == other.z
    }
}

impl Eq for Vector3<u32> {}

impl PartialEq for Vector3<f32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.z == other.z
    }
}

impl Eq for Vector3<f32> {}

#[derive(Debug, Clone, Copy)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Default + std::ops::AddAssign> Vector2<T> {
    pub fn new() -> Self {
        Self { x: T::default(), y: T::default() }
    }

    pub fn translation(&mut self, t: Self) {
        self.x += t.x;
        self.y += t.y;
    }
}

impl From<Vector2<u32>> for Vector2<f32> {
    fn from(val: Vector2<u32>) -> Self {
        Self {
            x: val.x as _,
            y: val.y as _,
        }
    }
}

impl From<Vector2<f32>> for Vector2<u32> {
    fn from(val: Vector2<f32>) -> Self {
        Self {
            x: val.x as _,
            y: val.y as _,
        }
    }
}

impl<T> From<Vector3<T>> for Vector2<T> {
    fn from(val: Vector3<T>) -> Self {
        Self {
            x: val.x,
            y: val.y
        }
    }
}

impl From<winit::dpi::PhysicalPosition<f32>> for Vector2<f32> {
    fn from(value: winit::dpi::PhysicalPosition<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y
        }
    }
}

impl From<winit::dpi::PhysicalPosition<u32>> for Vector2<u32> {
    fn from(value: winit::dpi::PhysicalPosition<u32>) -> Self {
        Self {
            x: value.x,
            y: value.y
        }
    }
}

impl PartialEq for Vector2<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Vector2<u32> {}

impl PartialEq for Vector2<f32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Vector2<f32> {}

pub fn cast_slice<A: Sized, B: Sized>(p: &[A]) -> Result<&[B], Error> {
    if align_of::<B>() > align_of::<A>()
        && (p.as_ptr() as *const () as usize) % align_of::<B>() != 0 {
        return Err(Error::PointersHaveDifferentAlignmnet);
    }
    unsafe {
        let len = size_of_val::<[A]>(p) / size_of::<B>();
        Ok(core::slice::from_raw_parts(p.as_ptr() as *const B, len))
    }
}

pub fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

pub fn _cos(x: f32, y: f32) -> f32 {
    let hyp = (x*x + y*y).sqrt();
    (x / hyp).abs()
}
