use shared::{Matrix4x4, Size, Vector2};

use crate::app::DEFAULT_SCREEN_SIZE;

use super::{
    cast_slice,
    Element,
    Gpu,
    RenderComponentSource,
    TextureData,
    TextureDataSource,
};

const INITIAL_CAPACITY: u64 = 1024 * 4;

#[derive(Debug)]
pub(crate) struct Buffer<T> {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) data: Vec<T>,
    label: String,
}

impl<T> Buffer<T> {
    fn uniform(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, wgpu::BufferUsages::UNIFORM, 1, label)
    }

    fn storage(device: &wgpu::Device, label: &str) -> Self {
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

    fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
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

    fn push(&mut self, data: T) {
        self.data.push(data);
    }

    pub(crate) fn update<F: FnMut(&mut T)>(&mut self, index: usize, mut f: F) {
        f(&mut self.data[index])
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

pub(crate) struct Gfx {
    pub(crate) elements: Buffer<Element>,
    pub(crate) transforms: Buffer<Matrix4x4>,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) textures: Vec<TextureData>,
}

impl Gfx {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let textures = vec![];
        let elements = Buffer::<Element>::storage(device, "element");
        let transforms = Buffer::<Matrix4x4>::storage(device, "transforms");
        let bind_group = Self::bind_group(device, &[
            elements.bind_group_entry(0),
            transforms.bind_group_entry(1),
        ]);

        Self { elements, transforms, bind_group, textures }
    }

    pub(crate) fn register(
        &mut self,
        gpu: &Gpu,
        maybe_pixel: Option<&impl TextureDataSource>,
        render_component: &impl RenderComponentSource,
    ) {
        let window_size: Size<f32> = gpu.size().into();

        let mut element = render_component.element();
        let transform = render_component.transform(window_size);
        let transform_id = self.transforms.len() as u32;
        element.transform_id = transform_id;

        self.push_texture(gpu, maybe_pixel, &mut element);
        self.transforms.push(transform);
        self.elements.push(element);
    }

    fn push_texture(
        &mut self,
        gpu: &Gpu,
        maybe_pixel: Option<&impl TextureDataSource>,
        element: &mut Element
    ) {
        if let Some(pixel) = maybe_pixel {
            let texture_id = self.textures.len() as i32;
            element.texture_id = texture_id;
            let texture_data = TextureData::new(gpu, pixel);
            self.textures.push(texture_data);
        } else {
            element.texture_id = -1;
        }
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

    pub(crate) fn indices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let mut indices = vec![];
        for _ in 0..self.count() {
            indices.extend_from_slice(&Indices::new());
        }
        
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices buffer"),
            contents: cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    pub(crate) fn vertices(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        let mut vertices = vec![];
        for _ in 0..self.count() {
            vertices.extend_from_slice(&Vertices::new());
        }

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
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

    pub(crate) fn vertice_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<Vector2<f32>>() as u64,
                    shader_location: 1,
                },
            ],
        }
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

pub(crate) struct Screen {
    transform: Buffer<Matrix4x4>,
    scaler: Buffer<Size<f32>>,
    pub(crate) bind_group: wgpu::BindGroup,
    initial_size: Size<f32>,
    initialized: bool,
}

impl Screen {
    pub(crate) fn new(device: &wgpu::Device, initial_size: Size<f32>) -> Self {
        let mut transform = Buffer::uniform(device, "screen_transform");
        let mut scaler = Buffer::uniform(device, "screen_scaler");
        transform.push(Matrix4x4::IDENTITY);
        scaler.push(DEFAULT_SCREEN_SIZE.into());
        let bind_group = Self::bind_group(device, &[
            transform.bind_group_entry(0),
            scaler.bind_group_entry(1)
        ]);
        Self {
            scaler,
            transform,
            bind_group,
            initial_size,
            initialized: false,
        }
    }

    pub(crate) fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.transform.write(device, queue);
        if !self.initialized {
            self.scaler.write(device, queue);
        }
        self.initialized = true;
    }

    pub(crate) fn initial_size(&self) -> Size<f32> {
        self.initial_size
    }

    pub(crate) fn update_transform<F: FnMut(&mut Matrix4x4)>(&mut self, f: F) {
        self.transform.update(0, f);
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                Buffer::<Matrix4x4>::uniform_bind_group_layout_entry(0),
                Buffer::<Size<f32>>::uniform_bind_group_layout_entry(1),
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        entries: &[wgpu::BindGroupEntry]
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &Self::bind_group_layout(device),
            entries,
        })
    }
}

// .....................................

#[derive(Debug, Clone)]
pub(crate) struct Indices<'a>(&'a [u32]);

impl std::ops::Deref for Indices<'_> {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Indices<'_> {
    const fn new() -> Self {
        Self(&[0, 1, 2, 2, 3, 0])
    }

    // const fn three() -> Self {
    //     Self(&[0, 1, 2])
    // }
}

// ....................................

#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    _pos: Vector2<f32>,
    _uv: Vector2<f32>,
}

// ....................................

pub(crate) struct Vertices([Vertex; 4]);

impl std::ops::Deref for Vertices {
    type Target = [Vertex];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for Vertices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl Vertices {
    const VERTICES: Self = Self ([
        Vertex { _pos: Vector2::new( -1.0,  1.0 ), _uv: Vector2::new( 0.0, 0.0 ) },
        Vertex { _pos: Vector2::new( -1.0, -1.0 ), _uv: Vector2::new( 0.0, 1.0 ) },
        Vertex { _pos: Vector2::new(  1.0, -1.0 ), _uv: Vector2::new( 1.0, 1.0 ) },
        Vertex { _pos: Vector2::new(  1.0,  1.0 ), _uv: Vector2::new( 1.0, 0.0 ) },
    ]);

    pub(crate) fn new() -> Self {
        // let s = size / DEFAULT_SCALER / 2.;
        // Self::VERTICES.adjust(s.width(), s.height())
        Self::VERTICES
    }

    pub(crate) fn as_slice(&self) -> &[Vertex] {
        self.0.as_slice()
    }

    // fn adjust(mut self, sw: f32, sh: f32) -> Self {
    //     self.iter_mut().for_each(|vertex| {
    //         vertex._pos.mul_x(sw);
    //         vertex._pos.mul_y(sh);
    //     });
    //     self
    // }
}

impl std::fmt::Debug for Vertices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let len = self.0.len();
        for i in 0..len {
            let pos = self.0[i]._pos;
            if i == len - 1 {
                s.push_str(format!("{i}: {pos:?}").as_str());
            } else {
                s.push_str(format!("{i}: {pos:?}\n").as_str());
            }
        }
        write!(f, "{s}")
    }
}
