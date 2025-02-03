use std::marker::PhantomData;

use math::{Matrix, Size, Vector4};
use wgpu::util::DeviceExt;

use crate::storage::cast_slice;
use crate::{Color, NodeId, Rgba};
use crate::shapes::Shape;
use super::TextureData;

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub count: u32,
    usage: wgpu::BufferUsages,
    len: usize,
    label: String,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> {
    // pub fn v(device: &wgpu::Device, data: &[u8], label: impl Into<String>) -> Self {
    //     Self::new(device, wgpu::BufferUsages::VERTEX, data, label)
    // }

    pub fn i(device: &wgpu::Device, data: &[u8], label: impl Into<String>) -> Self {
        Self::new(device, wgpu::BufferUsages::INDEX, data, label)
    }

    pub fn u(device: &wgpu::Device, data: &[u8], label: impl Into<String>) -> Self {
        Self::new(device, wgpu::BufferUsages::UNIFORM, data, label)
    }

    // pub fn s(device: &wgpu::Device, data: &[u8], label: impl Into<String>) -> Self {
    //     Self::new(device, wgpu::BufferUsages::STORAGE, data, label)
    // }

    fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[u8], label: impl Into<String>) -> Self {
        let len = data.len();
        let label: String = label.into();
        let count = (len / size_of_val(&usage)) as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            contents: data,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            count,
            usage,
            len,
            label,
            _phantom: PhantomData,
        }
    }

    pub fn slice(&self) -> wgpu::BufferSlice { self.buffer.slice(..) }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        if data.len() != self.len {
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(self.label.as_str()),
                contents: data,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
            });
            self.len = data.len();
            self.count = (self.len / size_of_val(&self.usage)) as u32;
        } else {
            queue.write_buffer(&self.buffer, 0, data)
        }
    }
}

pub struct Gfx {
    pub i: Buffer<Vec<u32>>,
    pub u: Buffer<Matrix<Vector4<f32>, 4>>,
    pub t: TextureData,
    pub bg: wgpu::BindGroup,
}

impl Gfx {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
        color: Color<Rgba<u8>, u8>,
        shape: &Shape,
        node_id: NodeId
    ) -> Self {
        let i = shape.i_buffer(node_id, device);
        let u = shape.u_buffer(node_id, device);
        let t = TextureData::new(device, queue, color);
        let bg = t.bind_group(device, bg_layout, &u.buffer);

        Self { i, u, t, bg }
    }

    pub fn _bind_group_layout(
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
}

pub struct Screen {
    pub initial_size: Size<u32>,
    pub transform: Matrix<Vector4<f32>, 4>,
    pub buffer: Buffer<Matrix<Vector4<f32>, 4>>,
    pub bind_group: wgpu::BindGroup,
}

impl Screen {
    pub fn new(device: &wgpu::Device, initial_size: Size<u32>) -> Self {
        let transform = Matrix::IDENTITY;
        let buffer = Buffer::u(device, cast_slice(transform.data()), "screen");
        let bind_group = Self::bind_group(device, &buffer.buffer);
        Self {
            initial_size,
            transform,
            buffer,
            bind_group
        }
    }

    pub fn update<F: FnMut(&mut Matrix<Vector4<f32>, 4>)>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut f: F
    ) {
        f(&mut self.transform);
        let data = self.transform.data();
        self.buffer.update(device, queue, cast_slice(data));
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                },
            ],
        })
    }
}
