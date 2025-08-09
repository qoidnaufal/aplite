use aplite_types::{Rect, Vec2f};

use crate::buffer::Buffer;

pub(crate) struct MeshBuffer {
    pub(crate) indices: Buffer<u32>,
    pub(crate) vertices: Buffer<Vertex>,
    pub(crate) offset: u64,
}

impl MeshBuffer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            indices: Buffer::new(device, 1024 * 6, wgpu::BufferUsages::INDEX, "index"),
            vertices: Buffer::new(device, 1024 * 4, wgpu::BufferUsages::VERTEX, "vertex"),
            offset: 0,
        }
    }

    pub(crate) fn indices_slice(&self) -> wgpu::BufferSlice<'_> {
        self.indices.slice(0..self.offset * 6)
    }

    pub(crate) fn vertices_slice(&self) -> wgpu::BufferSlice<'_> {
        self.vertices.slice(0..self.offset * 4)
    }

    pub(crate) fn vertice_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<Vec2f>() as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: size_of::<Vec2f>() as u64 * 2,
                    shader_location: 2,
                }
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Indices([u32; 6]);

#[derive(Clone, Copy)]
pub struct Vertices([Vertex; 4]);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    _pos: Vec2f,
    uv: Vec2f,
    id: u32,
}

impl Indices {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self([0, 1, 2, 2, 3, 0])
    }

    /// if need_adjust, the offset will be add by 4
    /// otherwise, each index will be add_assigned directly with the offset
    pub(crate) fn with_offset(mut self, mut offset: u32, need_adjust: bool) -> Self {
        if need_adjust { offset *= 4 }
        self.iter_mut().for_each(|i| *i += offset);
        self
    }

    pub(crate) const fn as_slice(&self) -> &[u32] {
        &self.0
    }
}

impl std::ops::Deref for Indices {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Indices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Vertices {
    const VERTICES: Self = Self ([
        Vertex { _pos: Vec2f::new(-1.0,  1.0), uv: Vec2f::new(0.0, 0.0), id: 0 },
        Vertex { _pos: Vec2f::new(-1.0, -1.0), uv: Vec2f::new(0.0, 1.0), id: 0 },
        Vertex { _pos: Vec2f::new( 1.0, -1.0), uv: Vec2f::new(1.0, 1.0), id: 0 },
        Vertex { _pos: Vec2f::new( 1.0,  1.0), uv: Vec2f::new(1.0, 0.0), id: 0 },
    ]);

    #[inline(always)]
    pub const fn new() -> Self { Self::VERTICES }

    pub fn with_uv(mut self, uv: Rect) -> Self {
        self.set_uv(uv);
        self
    }

    pub fn set_uv(&mut self, uv: Rect) {
        let l = uv.x;
        let r = uv.max_x();
        let t = uv.y;
        let b = uv.max_y();

        self.iter_mut().for_each(|v| {
            if v.uv.x == 0.0 { v.uv.x = l } else { v.uv.x = r }
            if v.uv.y == 0.0 { v.uv.y = t } else { v.uv.y = b }
        });
    }

    pub fn with_id(mut self, id: u32) -> Self {
        self.set_id(id);
        self
    }

    pub fn set_id(&mut self, id: u32) {
        self.iter_mut().for_each(|v| v.id = id);
    }

    #[inline(always)]
    pub(crate) const fn as_slice(&self) -> &[Vertex] {
        self.0.as_slice()
    }
}

impl std::fmt::Debug for Vertices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let len = self.0.len();
        for i in 0..len {
            let pos = self.0[i]._pos;
            let uv = self.0[i].uv;
            if i == len - 1 {
                s.push_str(format!("{i}: {pos:?} | {uv:?}").as_str());
            } else {
                s.push_str(format!("{i}: {pos:?} | {uv:?}\n").as_str());
            }
        }
        write!(f, "{s}")
    }
}

impl std::ops::Deref for Vertices {
    type Target = [Vertex];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for Vertices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

// struct Indx {
//     i: Vec<u32>,
// }

// struct Vrtx {
//     v: Vec<Vertex>,
// }

// struct Mesh {
//     indices: Vec<Indx>,
//     vertices: Vec<Vrtx>,
//     indices_count: u64,
// }
