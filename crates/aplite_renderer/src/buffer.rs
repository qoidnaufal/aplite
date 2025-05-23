fn cast_slice<SRC: Sized, DST: Sized>(src: &[SRC]) -> &[DST] {
    let len = size_of_val(src);
    unsafe { core::slice::from_raw_parts(src.as_ptr() as *const DST, len) }
}

pub(crate) struct Buffer<T> {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Buffer<T> {
    pub(crate) fn new(
        device: &wgpu::Device,
        capacity: u64,
        usage: wgpu::BufferUsages,
        label: &'static str
    ) -> Self {
        let size = size_of::<T>() as u64 * capacity;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            usage,
            label,
            _phantom: Default::default(),
        }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, offset: u64, data: &[T]) -> bool {
        let len = data.len() as u64;
        let realloc = offset + len > self.capacity;

        if realloc {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: size_of::<T>() as u64 * len,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacity = len;
        }

        let offset = offset * size_of::<T>() as u64;
        queue.write_buffer(&self.buffer, offset, cast_slice(data));

        realloc
    }

    pub(crate) fn bind_group_layout_entry(ty: wgpu::BufferBindingType, binding: u32) -> wgpu::BindGroupLayoutEntry {
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

    pub(crate) fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
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

// pub(crate) struct Uniform<T: Copy> {
//     buffer: wgpu::Buffer,
//     data: T,
// }

// impl<T: Copy> Uniform<T> {
//     pub(crate) fn new(device: &wgpu::Device, data: T, label: &str) -> Self {
//         let buffer = device.create_buffer(&wgpu::BufferDescriptor {
//             label: Some(label),
//             size: size_of::<T>() as u64,
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//             mapped_at_creation: false,
//         });
//         Self {
//             buffer,
//             data,
//         }
//     }

//     pub(crate) const fn data(&self) -> T { self.data }

//     pub(crate) fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
//         wgpu::BindGroupLayoutEntry {
//             binding,
//             visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
//             ty: wgpu::BindingType::Buffer {
//                 ty: wgpu::BufferBindingType::Uniform,
//                 has_dynamic_offset: false,
//                 min_binding_size: None,
//             },
//             count: None,
//         }
//     }

//     pub(crate) fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
//         wgpu::BindGroupEntry {
//             binding,
//             resource: self.buffer.as_entire_binding(),
//         }
//     }

//     pub(crate) fn write(&mut self, queue: &wgpu::Queue) {
//         queue.write_buffer(&self.buffer, 0, cast_slice(&[self.data]));
//     }

//     pub(crate) fn update(&mut self, f: impl Fn(&mut T)) {
//         f(&mut self.data)
//     }
// }

// const INITIAL_CAPACITY: usize = 1024;

// pub(crate) struct Storage<T> {
//     pub(crate) buffer: wgpu::Buffer,
//     // FIXME: maybe don't need this?
//     pub(crate) data: Vec<T>,
//     capacity: usize,
//     label: String,
// }

// impl<T> Storage<T> {
//     pub(crate) fn new(
//         device: &wgpu::Device,
//         label: &'static str
//     ) -> Self {
//         let size = size_of::<T>() * INITIAL_CAPACITY;
//         let buffer = device.create_buffer(&wgpu::BufferDescriptor {
//             label: Some(label),
//             size: size as u64,
//             usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
//             mapped_at_creation: false,
//         });
//         Self {
//             buffer,
//             data: vec![],
//             label: label.into(),
//             capacity: INITIAL_CAPACITY,
//         }
//     }

//     pub(crate) fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
//         wgpu::BindGroupLayoutEntry {
//             binding,
//             visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
//             ty: wgpu::BindingType::Buffer {
//                 ty: wgpu::BufferBindingType::Storage { read_only: true },
//                 has_dynamic_offset: false,
//                 min_binding_size: None,
//             },
//             count: None,
//         }
//     }

//     pub(crate) fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry {
//         wgpu::BindGroupEntry {
//             binding,
//             resource: self.buffer.as_entire_binding(),
//         }
//     }

//     pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
//         let realloc = self.data.len() > self.capacity;
//         if realloc {
//             self.capacity = self.data.len().next_power_of_two();
//             let usage = self.buffer.usage();
//             self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
//                 label: Some(self.label.as_str()),
//                 size: (self.capacity * size_of::<T>()) as u64,
//                 usage,
//                 mapped_at_creation: false,
//             });
//         }
//         queue.write_buffer(&self.buffer, 0, cast_slice(&self.data));
//         realloc
//     }

//     pub(crate) fn push(&mut self, data: T) { self.data.push(data) }

//     pub(crate) fn update<F: FnMut(&mut T)>(&mut self, index: usize, mut f: F) {
//         f(&mut self.data[index])
//     }

//     pub(crate) fn len(&self) -> usize { self.data.len() }
// }
