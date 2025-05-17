use shared::{Matrix4x4, Rect, Vector2};

use super::element::Element;
use super::buffer::Storage;
use super::util::{
    cast_slice,
    Indices,
    Vertex,
    Vertices,
};

pub(crate) struct Gfx {
    pub(crate) elements: Storage<Element>,
    pub(crate) transforms: Storage<Matrix4x4>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) uvs: Vec<Rect<f32>>,
}

impl Gfx {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let elements = Storage::<Element>::new(device, "element");
        let transforms = Storage::<Matrix4x4>::new(device, "transforms");
        let bind_group = Self::bind_group(device, &[
            elements.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);
        let uvs = vec![];

        Self { elements, transforms, bind_group, uvs }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let mut realloc = false;
        realloc |= self.elements.write(device, queue);
        realloc |= self.transforms.write(device, queue);

        if realloc {
            self.bind_group = Self::bind_group(device, &[
                self.elements.bind_group_entry(0),
                self.transforms.bind_group_entry(1),
            ]);
        }

        realloc
    }

    pub(crate) fn indices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let mut indices = vec![];
        for _ in 0..self.count() { indices.extend_from_slice(&Indices::new()) }
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices buffer"),
            contents: cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub(crate) fn vertices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let mut vertices = vec![];
        for _ in 0..self.count() { vertices.extend_from_slice(&Vertices::new()) }

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub(crate) fn instances(&self,device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(&(0..self.count() as u32).collect::<Vec<_>>()),
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

    pub(crate) fn instance_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<u32>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 0,
                    shader_location: 2,
                }
            ],
        }
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gfx bind group layout"),
            entries: &[
                Storage::<Element>::bind_group_layout_entry(0),
                Storage::<Matrix4x4>::bind_group_layout_entry(1),
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupEntry<'_>],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gfx bind group"),
            layout: &Self::bind_group_layout(device),
            entries,
        })
    }

    pub(crate) fn count(&self) -> usize {
        self.elements.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.elements.data.is_empty()
    }
}
