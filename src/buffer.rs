use wgpu::util::DeviceExt;

use crate::{error::Error, shapes::Vertex, types::cast_slice};

pub struct Buffer {
    pub v: wgpu::Buffer,
    pub i: wgpu::Buffer,
}

impl Buffer {
    pub fn new(device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<u16>) -> Result<Self, Error> {
        let v = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(&vertices)?,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let i = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: cast_slice(&indices)?,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        Ok(Self { v, i })
    }
}

