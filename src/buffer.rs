use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use crate::{shapes::Vertex, texture::TextureData};

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub len: u32,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[u8]) -> Self {
        let len = (data.len() / size_of_val(&usage)) as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("{} vertex buffer"),
            contents: data,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            len,
            _phantom: PhantomData,
        }
    }

    pub fn slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(..)
    }

    pub fn update(&self, queue: &wgpu::Queue, offset: usize, data: &[u8]) {
        queue.write_buffer(
            &self.buffer,
            offset as wgpu::BufferAddress,
            data,
        );
    }
}

#[derive(Default)]
pub struct Gfx {
    pub v_buffer: Vec<Buffer<Vertex>>,
    pub i_buffer: Vec<Buffer<u32>>,
    pub textures: Vec<TextureData>,
}
