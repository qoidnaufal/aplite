use aplite_types::{Rect, Vector2};

use crate::buffer::Buffer;

#[derive(Debug, Clone)]
pub(crate) struct Indices([u32; 6]);

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
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Vertex {
    _pos: Vector2<f32>,
    uv: Vector2<f32>,
    id: u32,
}

pub(crate) struct Vertices([Vertex; 4]);

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

impl Vertices {
    const VERTICES: Self = Self ([
        Vertex { _pos: Vector2::new( -1.0,  1.0 ), uv: Vector2::new( 0.0, 0.0 ), id: 0 },
        Vertex { _pos: Vector2::new( -1.0, -1.0 ), uv: Vector2::new( 0.0, 1.0 ), id: 0 },
        Vertex { _pos: Vector2::new(  1.0, -1.0 ), uv: Vector2::new( 1.0, 1.0 ), id: 0 },
        Vertex { _pos: Vector2::new(  1.0,  1.0 ), uv: Vector2::new( 1.0, 0.0 ), id: 0 },
    ]);

    #[inline(always)]
    pub(crate) const fn new() -> Self { Self::VERTICES }

    pub(crate) fn with_uv(mut self, rect: Rect<f32>) -> Self {
        let l = rect.l() as f32;
        let r = rect.r() as f32;
        let t = rect.t() as f32;
        let b = rect.b() as f32;

        self.iter_mut().for_each(|v| {
            if v.uv.x() == 0.0 { v.uv.set_x(l) } else { v.uv.set_x(r) }
            if v.uv.y() == 0.0 { v.uv.set_y(t) } else { v.uv.set_y(b) }
        });

        self
    }

    fn with_id(mut self, id: u32) -> Self {
        self.iter_mut().for_each(|v| v.id = id);
        self
    }

    #[inline(always)]
    pub(crate) fn as_slice(&self) -> &[Vertex] {
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

pub(crate) struct MeshBuffer {
    pub(crate) indices: Buffer<u32>,
    pub(crate) vertices: Buffer<Vertex>,
    pub(crate) offset: u64,

    // FIXME: this is for rebuild or expand only?
    pub(crate) uvs: Vec<Rect<f32>>,
}

impl MeshBuffer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            indices: Buffer::new(device, 1024 * 6, wgpu::BufferUsages::INDEX, "index"),
            vertices: Buffer::new(device, 1024 * 4, wgpu::BufferUsages::VERTEX, "vertex"),
            offset: 0,
            uvs: Vec::with_capacity(1024),
        }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.uvs.is_empty() { return; }

        let mut idx = vec![];
        let mut vtx = vec![];
        for i in 0..self.uvs.len() {
            let uv = self.uvs[i];
            idx.extend_from_slice(&Indices::new().with_offset(i as _, true));
            vtx.extend_from_slice(&Vertices::new().with_uv(uv).with_id(i as _));
        }

        self.indices.write(device, queue, self.offset, &idx);
        self.vertices.write(device, queue, self.offset, &vtx);

        self.offset += self.uvs.len() as u64;
        self.uvs.clear();
    }

    pub(crate) fn vertice_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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
                    offset: size_of::<Vector2<f32>>() as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: size_of::<Vector2<f32>>() as u64 * 2,
                    shader_location: 2,
                }
            ],
        }
    }
}
