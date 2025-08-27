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

    // pub(crate) fn map_buffer(&self, device: &wgpu::Device) -> bool {
    //     use std::sync::{Arc, OnceLock};

    //     let ready = Arc::new(OnceLock::<Result<(), wgpu::BufferAsyncError>>::new());
    //     let send = ready.clone();

    //     self.buffer.map_async(
    //         wgpu::MapMode::Write,
    //         0..,
    //         move |r| send.set(r).unwrap()
    //     );

    //     device.poll(wgpu::PollType::Wait).unwrap();
    //     ready.get().unwrap().is_ok()
    // }

    // pub(crate) fn map_write(
    //     &self,
    //     offset: u64,
    //     data: &[T],
    // ) {
    //     let start = offset * size_of::<T>() as u64;
    //     let end = start + (data.len() * size_of::<T>()) as u64;
    //     self.buffer
    //         .get_mapped_range_mut(start..end)
    //         .copy_from_slice(cast_slice(data));
    // }

    // pub(crate) fn unmap(&self) {
    //     self.buffer.unmap();
    // }

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

            let mut resize_encoder = device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("buffer realoc")
                }
            );

            resize_encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, None);
            resize_encoder.finish();

            self.buffer.destroy();
            self.buffer = new_buffer;
        }

        queue.write_buffer(
            &self.buffer,
            offset * size_of::<T>() as u64,
            cast_slice(data)
        );

        realloc
    }

    // pub(crate) fn write_with_belt(
    //     &mut self,
    //     device: &wgpu::Device,
    //     staging_belt: &mut wgpu::util::StagingBelt,
    //     encoder: &mut wgpu::CommandEncoder,
    //     offset: u64,
    //     data: &[T],
    // ) -> bool {
    //     let current_size = self.buffer.size();
    //     let realloc = (offset + data.len() as u64) * size_of::<T>() as u64 > current_size;

    //     if realloc {
    //         let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    //             label: Some(std::any::type_name::<T>()),
    //             size: current_size * 2,
    //             usage: self.buffer.usage(),
    //             mapped_at_creation: false,
    //         });

    //         let mut resize_encoder = device.create_command_encoder(
    //             &wgpu::CommandEncoderDescriptor {
    //                 label: Some("buffer realoc")
    //             }
    //         );

    //         resize_encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, None);
    //         resize_encoder.finish();

    //         self.buffer.destroy();
    //         self.buffer = new_buffer;
    //     }

    //     staging_belt.write_buffer(
    //         encoder,
    //         &self.buffer,
    //         offset * size_of::<T>() as u64,
    //         std::num::NonZeroU64::new(data.len() as u64 * size_of::<T>() as u64)
    //             .expect("Item should not have a zero size"),
    //         device
    //     ).copy_from_slice(cast_slice(data));

    //     realloc
    // }

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
