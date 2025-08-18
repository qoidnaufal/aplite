fn cast_slice<SRC: Sized, DST: Sized>(src: &[SRC]) -> &[DST] {
    let len = size_of_val(src);
    unsafe { core::slice::from_raw_parts(src.as_ptr() as *const DST, len) }
}

pub(crate) struct Buffer<T> {
    buffer: wgpu::Buffer,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Buffer<T> {
    pub(crate) fn new(
        device: &wgpu::Device,
        capacity: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let size = size_of::<T>() as u64 * capacity;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(std::any::type_name::<T>()),
            size,
            usage: usage
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            _phantom: Default::default(),
        }
    }

    pub(crate) fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        offset: u64,
        data: &[T],
    ) -> bool {
        let current_size = self.buffer.size();
        let realloc = (offset + data.len() as u64) * size_of::<T>() as u64 > current_size;

        if realloc {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(std::any::type_name::<T>()),
                size: current_size * 2,
                usage: self.buffer.usage(),
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("buffer realoc")
                }
            );

            encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, None);
            encoder.finish();

            self.buffer = new_buffer;
        }

        let offset = offset * size_of::<T>() as u64;
        queue.write_buffer(&self.buffer, offset, cast_slice(data));

        realloc
    }

    pub(crate) fn bind_group_layout_entry(
        ty: wgpu::BufferBindingType,
        binding: u32
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub(crate) fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }

    pub(crate) fn slice(&self, range: std::ops::Range<u64>) -> wgpu::BufferSlice<'_> {
        let start = range.start * size_of::<T>() as u64;
        let end = range.end * size_of::<T>() as u64;
        self.buffer.slice(start..end)
    }
}
