use util::{cast_slice, Matrix, Matrix4x4, Size};

use crate::shapes::Shape;

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
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

    // pub fn slice<'a, R: RangeBounds<u64>>(&'a self, range: R) -> wgpu::BufferSlice<'a> {
    //     self.buffer.slice(range)
    // }

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
    indices: Vec<u32>,
}

impl Gfx {
    pub fn new(
        device: &wgpu::Device,
    ) -> Self {
        let indices = vec![];
        let shapes = Buffer::<Shape>::storage(device, "shapes");
        let transforms = Buffer::<Matrix4x4>::storage(device, "transforms");
        let bind_group = Self::bind_group(device, &[
            shapes.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);

        Self { indices, shapes, transforms, bind_group }
    }

    pub fn push(&mut self, mut shape: Shape, window_size: Size<u32>) {
        let transform_idx = self.transforms.data.len() as u32;
        let transform = shape.get_transform(window_size);
        shape.transform = transform_idx;
        self.indices.extend_from_slice(&*shape.indices());
        self.shapes.push(shape);
        self.transforms.push(transform);
    }

    // pub fn total_indices_len(&self) -> usize {
    //     self.indices.len()
    // }

    pub fn indices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices buffer"),
            contents: cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        })
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

pub struct Uniform {
    pub initial_size: Size<u32>,
    pub screen: Buffer<Matrix4x4>,
    pub bind_group: wgpu::BindGroup,
}

impl Uniform {
    pub fn new(device: &wgpu::Device, initial_size: Size<u32>) -> Self {
        let transform = Matrix::IDENTITY;
        let mut screen = Buffer::uniform(device, "screen");
        screen.push(transform);
        let sampler = sampler(device);
        let bind_group = Self::bind_group(device, &screen, &sampler);
        Self {
            initial_size,
            screen,
            bind_group
        }
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.screen.write(device, queue);
    }

    pub fn update<F: FnMut(&mut Matrix4x4)>(&mut self, f: F) {
        self.screen.update(0, f);
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                Buffer::<Matrix4x4>::uniform_bind_group_layout_entry(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
        })
    }

    pub fn bind_group(
        device: &wgpu::Device,
        screen: &Buffer<Matrix4x4>,
        sampler: &wgpu::Sampler
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                screen.bind_group_entry(0),
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                }
            ],
        })
    }
}

fn sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
