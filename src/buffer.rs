use wgpu::util::DeviceExt;

pub struct Buffer {
    pub v: wgpu::Buffer,
    pub i: wgpu::Buffer,
}

impl Buffer {
    pub fn new(device: &wgpu::Device, data: &[u8], indices: &[u8]) -> Self {
        let v = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let i = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: indices,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        Self { v, i }
    }

    pub fn update(&self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(
            &self.v,
            0,
            data,
        );
    }
}

