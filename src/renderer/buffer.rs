use util::{cast_slice, Matrix4x4, Size};

use crate::shapes::{Shape, ShapeConfig, ShapeKind};

use super::TextureData;

const INITIAL_CAPACITY: u64 = 1024 * 4;

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub data: Vec<T>,
    usage: wgpu::BufferUsages,
    label: String,
}

impl<T> Buffer<T> {
    pub fn uniform(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, wgpu::BufferUsages::UNIFORM, 1, label)
    }

    pub fn storage(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, wgpu::BufferUsages::STORAGE, INITIAL_CAPACITY, label)
    }

    fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, capacity: u64, label: &str) -> Self {
        let size = size_of::<T>() as u64 * capacity;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            usage,
            label: label.to_string(),
            data: vec![],
        }
    }

    fn storage_bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn uniform_bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let data_size = self.data.len() * size_of::<T>();
        let realloc = data_size as u64 > self.buffer.size();
        if realloc {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: self.buffer.size().next_power_of_two(),
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buffer, 0, cast_slice(&self.data));
        realloc
    }

    pub fn push(&mut self, data: T) {
        self.data.push(data);
    }

    pub fn update<F: FnMut(&mut T)>(&mut self, index: usize, mut f: F) {
        f(&mut self.data[index])
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct Gfx {
    pub shapes: Buffer<Shape>,
    pub transforms: Buffer<Matrix4x4>,
    pub bind_group: wgpu::BindGroup,
    pub textures: Vec<TextureData>,
    indices: Vec<u32>,
}

impl Gfx {
    pub fn new(
        device: &wgpu::Device,
    ) -> Self {
        let indices = vec![];
        let textures = vec![];
        let shapes = Buffer::<Shape>::storage(device, "shapes");
        let transforms = Buffer::<Matrix4x4>::storage(device, "transforms");
        let bind_group = Self::bind_group(device, &[
            shapes.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);

        Self { shapes, transforms, bind_group, indices, textures }
    }

    pub fn count(&self) -> usize {
        self.shapes.len()
    }

    pub fn push(&mut self, config: &ShapeConfig, window_size: Size<u32>, kind: ShapeKind) {
        let transform = config.get_transform(window_size);
        let transform_id = self.transforms.len() as u32;
        let shape = Shape::new(config, transform_id, kind);
        self.indices.extend_from_slice(&shape.indices());
        self.transforms.push(transform);
        self.shapes.push(shape);
    }

    pub fn push_texture(&mut self, texture_data: TextureData, config: &mut ShapeConfig) {
        let texture_id = self.textures.len() as i32;
        config.texture_id = texture_id;
        self.textures.push(texture_data);
    }

    pub fn indices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices buffer"),
            contents: cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub fn instance(&self,device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        let instance_data = (0..self.count() as u32).collect::<Vec<_>>();
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn instance_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut realloc = false;
        realloc |= self.shapes.write(device, queue);
        realloc |= self.transforms.write(device, queue);

        if realloc {
            self.bind_group = Self::bind_group(device, &[
                self.shapes.bind_group_entry(0),
                self.transforms.bind_group_entry(1),
            ]);
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gfx bind group layout"),
            entries: &[
                Buffer::<Shape>::storage_bind_group_layout_entry(0),
                Buffer::<Matrix4x4>::storage_bind_group_layout_entry(1),
            ],
        })
    }

    pub fn bind_group(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupEntry<'_>],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gfx bind group"),
            layout: &Self::bind_group_layout(device),
            entries,
        })
    }
}

pub struct Screen {
    initial_size: Size<u32>,
    pub buffer: Buffer<Matrix4x4>,
    pub bind_group: wgpu::BindGroup,
}

impl Screen {
    pub fn new(device: &wgpu::Device, initial_size: Size<u32>) -> Self {
        let transform = Matrix4x4::IDENTITY;
        let mut buffer = Buffer::uniform(device, "screen");
        buffer.push(transform);
        let bind_group = Self::bind_group(device, &buffer);
        Self {
            initial_size,
            buffer,
            bind_group
        }
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.buffer.write(device, queue);
    }

    pub fn initial_size(&self) -> Size<u32> {
        self.initial_size
    }

    pub fn update<F: FnMut(&mut Matrix4x4)>(&mut self, f: F) {
        self.buffer.update(0, f);
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                Buffer::<Matrix4x4>::uniform_bind_group_layout_entry(0),
            ],
        })
    }

    pub fn bind_group(
        device: &wgpu::Device,
        buffer: &Buffer<Matrix4x4>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                buffer.bind_group_entry(0),
            ],
        })
    }
}
