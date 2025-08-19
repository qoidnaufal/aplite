use aplite_types::{Rect, Size, Vec2f};

use crate::buffer::Buffer;

pub(crate) struct MeshBuffer {
    pub(crate) indices: Buffer<u32>,
    pub(crate) vertices: Buffer<Vertex>,
    pub(crate) offset: u64,
}

impl MeshBuffer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            indices: Buffer::new(device, 1024 * 4, wgpu::BufferUsages::INDEX),
            vertices: Buffer::new(device, 1024 * 4, wgpu::BufferUsages::VERTEX),
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
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32,
                    offset: 4 + size_of::<Vec2f>() as u64 * 2,
                    shader_location: 3,
                }
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Indices([u32; 4]);

#[derive(Clone, Copy)]
pub struct Vertices([Vertex; 4]);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pos: Vec2f,
    uv: Vec2f,
    id: u32,
    atlas: i32,
}

impl Indices {
    #[inline(always)]
    pub(crate) const fn new(offset: u32) -> Self {
        Self([
            1 + offset * 4,
            2 + offset * 4,
            0 + offset * 4,
            3 + offset * 4,
        ])
    }

    pub(crate) const fn as_slice(&self) -> &[u32] {
        &self.0
    }
}

impl Default for Vertices {
    fn default() -> Self {
        Self::VERTICES
    }
}

impl Vertices {
    const VERTICES: Self = Self([
        Vertex { pos: Vec2f::new(-1.0,  1.0), uv: Vec2f::new(0.0, 0.0), id: 0, atlas: -1 },
        Vertex { pos: Vec2f::new(-1.0, -1.0), uv: Vec2f::new(0.0, 1.0), id: 0, atlas: -1 },
        Vertex { pos: Vec2f::new( 1.0, -1.0), uv: Vec2f::new(1.0, 1.0), id: 0, atlas: -1 },
        Vertex { pos: Vec2f::new( 1.0,  1.0), uv: Vec2f::new(1.0, 0.0), id: 0, atlas: -1 },
    ]);

    pub fn new(rect: &Rect, uv: Rect, screen: Size, id: u32, atlas: i32) -> Self {
        let sx = screen.width;
        let sy = screen.height;

        let min_x = (rect.x / sx) * 2.0 - 1.0;
        let min_y = 1.0 - (rect.y / sy) * 2.0;
        let max_x = (rect.max_x() / sx) * 2.0 - 1.0;
        let max_y = 1.0 - (rect.max_y() / sy) * 2.0;

        let l = uv.x;
        let r = uv.max_x();
        let t = uv.y;
        let b = uv.max_y();

        Self([
            Vertex { pos: Vec2f::new(min_x, min_y), uv: Vec2f::new(l, t), id, atlas },
            Vertex { pos: Vec2f::new(min_x, max_y), uv: Vec2f::new(l, b), id, atlas },
            Vertex { pos: Vec2f::new(max_x, max_y), uv: Vec2f::new(r, b), id, atlas },
            Vertex { pos: Vec2f::new(max_x, min_y), uv: Vec2f::new(r, t), id, atlas },
        ])
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
            let pos = self.0[i].pos;
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
