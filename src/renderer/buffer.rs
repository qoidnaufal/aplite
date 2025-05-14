use super::{gfx::Gfx, util::cast_slice};

const INITIAL_CAPACITY: usize = 1024;

#[derive(Debug)]
pub(crate) struct Storage<T> {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) data: Vec<T>,
    capacity: usize,
    label: String,
}

impl<T> Storage<T> {
    pub(crate) fn new(
        device: &wgpu::Device,
        label: &'static str
    ) -> Self {
        let size = size_of::<T>() * INITIAL_CAPACITY;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            data: vec![],
            label: label.into(),
            capacity: INITIAL_CAPACITY,
        }
    }

    pub(crate) fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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

    pub(crate) fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let realloc = self.data.len() > self.capacity;
        if realloc {
            self.capacity = self.data.len().next_power_of_two();
            let usage = self.buffer.usage();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (self.capacity * size_of::<T>()) as u64,
                usage,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buffer, 0, cast_slice(&self.data));
        realloc
    }

    pub(crate) fn push(&mut self, data: T) {
        self.data.push(data);
    }

    #[allow(unused)]
    pub(crate) fn update<F: FnMut(&mut T)>(&mut self, index: usize, mut f: F) {
        f(&mut self.data[index])
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }
}

pub(crate) struct Uniform<T: Copy> {
    buffer: wgpu::Buffer,
    data: T,
}

impl<T: Copy> Uniform<T> {
    pub(crate) fn new(device: &wgpu::Device, data: T, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size_of::<T>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            data,
        }
    }

    pub(crate) const fn data(&self) -> T { self.data }

    pub(crate) fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub(crate) fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }

    pub(crate) fn write(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, cast_slice(&[self.data]));
    }

    pub(crate) fn update(&mut self, f: impl Fn(&mut T)) {
        f(&mut self.data)
    }
}

pub(crate) enum MeshBuffer {
    Uninitialized,
    Initialized {
        indices: wgpu::Buffer,
        vertices: wgpu::Buffer,
        instances: wgpu::Buffer,
    }
}

impl MeshBuffer {
    pub(crate) fn init(&mut self, gfx: &Gfx, device: &wgpu::Device) {
        *self = Self::Initialized {
            indices: gfx.indices(device),
            vertices: gfx.vertices(device),
            instances: gfx.instances(device),
        }
    }

    #[inline(always)]
    pub(crate) fn is_uninit(&self) -> bool {
        matches!(self, Self::Uninitialized)
    }

    pub(crate) fn get_buffer(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, &wgpu::Buffer)> {
        match self {
            MeshBuffer::Uninitialized => None,
            MeshBuffer::Initialized {
                indices,
                vertices,
                instances
            } => Some((indices, vertices, instances)),
        }
    }
}
