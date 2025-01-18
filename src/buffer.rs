use std::marker::PhantomData;

use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub len: usize,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[u8]) -> Self {
        let len = data.len();
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
        assert!(data.len() <= self.len - offset);
        queue.write_buffer(
            &self.buffer,
            offset as wgpu::BufferAddress,
            data,
        );
    }
}
