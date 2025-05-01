use util::{cast_slice, Matrix4x4, Size};

use crate::properties::Shape;
use super::{Element, Gpu, IntoRenderComponent, IntoTextureData, TextureData};

const INITIAL_CAPACITY: u64 = 1024 * 4;

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub data: Vec<T>,
    label: String,
}

impl<T> Buffer<T> {
    pub(crate) fn uniform(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, wgpu::BufferUsages::UNIFORM, 1, label)
    }

    pub(crate) fn storage(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, wgpu::BufferUsages::STORAGE, INITIAL_CAPACITY, label)
    }

    fn new(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        capacity: u64,
        label: &str
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
            label: label.to_string(),
            data: vec![],
        }
    }

    fn storage_bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let data_size = self.data.len() * size_of::<T>();
        let realloc = data_size > self.buffer.size() as usize;
        if realloc {
            let usage = self.buffer.usage();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: self.buffer.size().next_power_of_two(),
                usage,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buffer, 0, cast_slice(&self.data));
        realloc
    }

    pub(crate) fn push(&mut self, data: T) {
        self.data.push(data);
    }

    pub(crate) fn update<F: FnMut(&mut T)>(&mut self, index: usize, mut f: F) {
        f(&mut self.data[index])
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct Gfx {
    pub(crate) elements: Buffer<Element>,
    pub(crate) transforms: Buffer<Matrix4x4>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) textures: Vec<TextureData>,
    indices: Vec<u32>,
}

impl Gfx {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let indices = vec![];
        let textures = vec![];
        let elements = Buffer::<Element>::storage(device, "element");
        let transforms = Buffer::<Matrix4x4>::storage(device, "transforms");
        let bind_group = Self::bind_group(device, &[
            elements.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);

        Self { elements, transforms, bind_group, indices, textures }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut realloc = false;
        realloc |= self.elements.write(device, queue);
        realloc |= self.transforms.write(device, queue);

        if realloc {
            self.bind_group = Self::bind_group(device, &[
                self.elements.bind_group_entry(0),
                self.transforms.bind_group_entry(1),
            ]);
        }
    }

    pub(crate) fn register(
        &mut self,
        gpu: &Gpu,
        maybe_pixel: Option<&impl IntoTextureData>,
        render_component: &impl IntoRenderComponent,
    ) {
        let window_size = gpu.size();
        let mut element = render_component.element();
        let transform = render_component.transform(window_size);
        let transform_id = self.transforms.len() as u32;
        element.transform_id = transform_id;
        self.push_texture(gpu, maybe_pixel, &mut element);
        self.indices.extend_from_slice(&element.indices());
        self.transforms.push(transform);
        self.elements.push(element);
    }

    fn push_texture(
        &mut self,
        gpu: &Gpu,
        maybe_pixel: Option<&impl IntoTextureData>,
        element: &mut Element
    ) {
        if let Some(pixel) = maybe_pixel {
            let texture_id = self.textures.len() as i32;
            element.texture_id = texture_id;
            let texture_data = TextureData::new(gpu, pixel);
            self.textures.push(texture_data);
        }
    }

    pub(crate) fn indices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices buffer"),
            contents: cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub(crate) fn instances(&self,device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(&(0..self.count() as u32).collect::<Vec<_>>()),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub(crate) fn instance_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<u32>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 0,
                    shader_location: 2,
                }
            ],
        }
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gfx bind group layout"),
            entries: &[
                Buffer::<Element>::storage_bind_group_layout_entry(0),
                Buffer::<Matrix4x4>::storage_bind_group_layout_entry(1),
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupEntry<'_>],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gfx bind group"),
            layout: &Self::bind_group_layout(device),
            entries,
        })
    }

    pub(crate) fn count(&self) -> usize {
        self.elements.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.elements.data.is_empty()
    }
}

// const DEFAULT_SCREEN_SIZE: Size<u32> = Size::new(1600, 1200);

pub(crate) struct Screen {
    transform: Buffer<Matrix4x4>,
    size: Buffer<Size<u32>>,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Screen {
    pub(crate) fn new(device: &wgpu::Device, initial_size: Size<u32>) -> Self {
        let mut transform = Buffer::uniform(device, "screen_transform");
        let mut size = Buffer::uniform(device, "screen_size");
        transform.push(Matrix4x4::IDENTITY);
        size.push(initial_size);
        let bind_group = Self::bind_group(device, &transform, &size);
        Self {
            size,
            transform,
            bind_group,
        }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.transform.write(device, queue);
        self.size.write(device, queue);
    }

    pub(crate) fn initial_size(&self) -> Size<u32> {
        self.size.data[0]
    }

    pub(crate) fn update_transform<F: FnMut(&mut Matrix4x4)>(&mut self, f: F) {
        self.transform.update(0, f);
    }

    // pub(crate) fn update_size<F: FnMut(&mut Size<u32>)>(&mut self, f: F) {
    //     self.size.update(0, f);
    // }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                Buffer::<Matrix4x4>::uniform_bind_group_layout_entry(0),
                Buffer::<Size<u32>>::uniform_bind_group_layout_entry(1),
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        transform: &Buffer<Matrix4x4>,
        size: &Buffer<Size<u32>>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                transform.bind_group_entry(0),
                size.bind_group_entry(1),
            ],
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Indices<'a>(&'a [u32]);

impl std::ops::Deref for Indices<'_> {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl From<Shape> for Indices<'_> {
    fn from(shape: Shape) -> Self {
        match shape {
            Shape::Circle => Self::rectangle(),
            Shape::Rect => Self::rectangle(),
            Shape::RoundedRect => Self::rectangle(),
            Shape::Triangle => Self::triangle(),
        }
    }
}

impl Indices<'_> {
    pub(crate) fn rectangle() -> Self {
        Self(&[0, 1, 2, 2, 3, 0])
    }

    pub(crate) fn triangle() -> Self {
        Self(&[4, 1, 2])
    }
}

// pub struct Vertices<'a>(&'a [Vector2<f32>]);
