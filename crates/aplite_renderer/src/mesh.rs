use aplite_types::Vector2;

use crate::util::cast_slice;

#[derive(Debug, Clone)]
pub(crate) struct Indices<'a>(&'a [u32]);

impl std::ops::Deref for Indices<'_> {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Indices<'_> {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Self(&[0, 1, 2, 2, 3, 0])
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    _pos: Vector2<f32>,
    _uv: Vector2<f32>,
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
        Vertex { _pos: Vector2::new( -1.0,  1.0 ), _uv: Vector2::new( 0.0, 0.0 ) },
        Vertex { _pos: Vector2::new( -1.0, -1.0 ), _uv: Vector2::new( 0.0, 1.0 ) },
        Vertex { _pos: Vector2::new(  1.0, -1.0 ), _uv: Vector2::new( 1.0, 1.0 ) },
        Vertex { _pos: Vector2::new(  1.0,  1.0 ), _uv: Vector2::new( 1.0, 0.0 ) },
    ]);

    #[inline(always)]
    pub(crate) const fn new() -> Self { Self::VERTICES }

    // pub(crate) fn with_uv(mut self, rect: Rect<f32>) -> Self {
    //     for i in 0..4 {
    //         if self.0[i]._uv.x() == 0.0 { self.0[i]._uv.set_x(rect.x()) }
    //         else { self.0[i]._uv.set_x(rect.x() + rect.width()) }

    //         if self.0[i]._uv.y() == 0.0 { self.0[i]._uv.set_y(rect.y()) }
    //         else { self.0[i]._uv.set_y(rect.y() + rect.width()) }
    //     }
    //     self
    // }

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
            let uv = self.0[i]._uv;
            if i == len - 1 {
                s.push_str(format!("{i}: {pos:?} | {uv:?}").as_str());
            } else {
                s.push_str(format!("{i}: {pos:?} | {uv:?}\n").as_str());
            }
        }
        write!(f, "{s}")
    }
}

pub(crate) enum MeshBuffer {
    Uninitialized,
    Initialized {
        indices: wgpu::Buffer,
        vertices: wgpu::Buffer,
    }
}

impl MeshBuffer {
    pub(crate) fn init(&mut self, device: &wgpu::Device, n: usize) {
        *self = Self::Initialized {
            indices: Self::indices(device, n),
            vertices: Self::vertices(device, n),
        }
    }

    #[inline(always)]
    pub(crate) fn is_uninit(&self) -> bool {
        matches!(self, Self::Uninitialized)
    }

    pub(crate) fn get_buffer(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer)> {
        match self {
            MeshBuffer::Uninitialized => None,
            MeshBuffer::Initialized { indices, vertices } => Some((indices, vertices))
        }
    }

    pub(crate) fn indices(device: &wgpu::Device, n: usize) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let mut indices = vec![];
        for _ in 0..n { indices.extend_from_slice(&Indices::new()) }
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices buffer"),
            contents: cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub(crate) fn vertices(device: &wgpu::Device, n: usize) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let mut vertices = vec![];
        for _ in 0..n { vertices.extend_from_slice(&Vertices::new()) }

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
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
            ],
        }
    }
}
