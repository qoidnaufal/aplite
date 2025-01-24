use std::{collections::HashMap, marker::PhantomData};

use wgpu::util::DeviceExt;

use crate::{shapes::Vertex, texture::TextureData, NodeId};

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub materials: u32,
    usage: wgpu::BufferUsages,
    len: usize,
    label: String,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[u8], node_id: NodeId) -> Self {
        let len = data.len();
        let label = format!("{node_id:?} buffer");
        let materials = (data.len() / size_of_val(&usage)) as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            contents: data,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            materials,
            usage,
            len,
            label,
            _phantom: PhantomData,
        }
    }

    pub fn slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(..)
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, offset: usize, data: &[u8]) {
        if data.len() > self.len {
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(self.label.as_str()),
                contents: data,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            queue.write_buffer(
                &self.buffer,
                offset as wgpu::BufferAddress,
                data,
            );
        }
    }
}

#[derive(Default)]
pub struct Gfx {
    pub v_buffer: HashMap<NodeId, Buffer<Vertex>>,
    pub i_buffer: HashMap<NodeId, Buffer<Vec<u32>>>,
    pub textures: HashMap<NodeId, TextureData>,
}
