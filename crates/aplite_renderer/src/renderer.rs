use std::sync::Arc;
use winit::window::Window;
use winit::dpi::PhysicalSize;
use aplite_types::{Matrix3x2, Rgba, Size, PaintRef};

use super::RenderError;
use super::InitiationError;

use crate::atlas::Atlas;
use crate::element::{Element, Shape};
use crate::screen::Screen;
use crate::storage::StorageBuffers;
use crate::mesh::{Indices, MeshBuffer, Vertices};
use crate::util::Sampler;

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    // FIXME: maybe separating these was good?
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    encoder: Option<wgpu::CommandEncoder>,
    target_texture: Option<wgpu::SurfaceTexture>,

    // FIXME: not needed?
    screen: Screen,

    // FIXME: merge these two into Scene?
    storage: [StorageBuffers; 3],
    mesh: [MeshBuffer; 3],

    atlas: Atlas,
    sampler: Sampler,
    current: usize,
    clear_color: Rgba<f32>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, InitiationError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await?;

        let surface_capabilites = surface.get_capabilities(&adapter);

        let format = surface_capabilites
            .formats
            .iter()
            .find(|f| matches!(f, wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb))
            .copied()
            .unwrap_or(surface_capabilites.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                ..Default::default()
            },
        ).await?;

        surface.configure(&device, &config);

        let scale_factor = window.scale_factor();
        let logical: winit::dpi::LogicalSize<f32> = size.to_logical(scale_factor);
        let screen_size = Size::new(logical.width, logical.height);

        let screen = Screen::new(&device, screen_size, scale_factor);
        let atlas = Atlas::new(&device, Size::new(2000., 2000.));
        let sampler = Sampler::new(&device);

        let storage = [
            StorageBuffers::new(&device),
            StorageBuffers::new(&device),
            StorageBuffers::new(&device),
        ];

        let mesh = [
            MeshBuffer::new(&device),
            MeshBuffer::new(&device),
            MeshBuffer::new(&device),
        ];

        Ok(Self {
            device,
            queue,
            surface,
            config,
            encoder: None,
            target_texture: None,
            storage,
            sampler,
            atlas,
            mesh,
            screen,
            current: 0,
            clear_color: Rgba::new(0.0, 0.0, 0.0, 0.0),
        })
    }

    #[inline(always)]
    pub const fn scale_factor(&self) -> f64 {
        self.screen.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.screen.scale_factor = scale_factor;
    }

    /// Corresponds to [`winit::dpi::LogicalSize<u32>`]
    /// This one will not be updated when the window is resized.
    /// Important to determine the transform of an [`Element`].
    pub fn screen_res(&self) -> Size {
        self.screen.screen_size()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        let logical: winit::dpi::LogicalSize<f32> = new_size.to_logical(self.scale_factor());
        let res = self.screen_res();
        let ns = Size::new(logical.width, logical.height);
        let scale = res / ns;
        let sx = scale.width;
        let sy = scale.height;
        let matrix = Matrix3x2::from_scale_translate(sx, sy, sx - 1.0, 1.0 - sy);

        self.screen.write(&self.device, &self.queue, matrix);
    }

    pub fn begin(&mut self) -> Result<(), RenderError> {
        let label = Some("render encoder");
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label });

        let target_texture = self.surface.get_current_texture()?;

        self.encoder = Some(encoder);
        self.target_texture = Some(target_texture);

        self.current = (self.current + 1) % 3;
        self.mesh[self.current].offset = 0;

        Ok(())
    }

    #[inline(always)]
    pub fn scene(&mut self) -> Scene<'_> {
        Scene {
            screen_res: self.screen_res(),
            device: &self.device,
            queue: &self.queue,
            storage: &mut self.storage[self.current],
            mesh: &mut self.mesh[self.current],
            atlas: &mut self.atlas,
            clear_color: &mut self.clear_color,
        }
    }

    pub fn encode(&mut self) {
        if self.mesh[self.current].offset == 0 { return }

        let view = &self.target_texture
            .as_ref()
            .map(|tt| tt.texture.create_view(&wgpu::TextureViewDescriptor::default()))
            .unwrap();

        let desc = wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(
                    wgpu::Color {
                        r: self.clear_color.r as f64,
                        g: self.clear_color.g as f64,
                        b: self.clear_color.b as f64,
                        a: self.clear_color.a as f64,
                    }
                ),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        };

        let encoder = self.encoder.as_mut().unwrap();

        self.atlas.update(&self.device, encoder);

        let buffers = &[MeshBuffer::vertice_layout()];

        let bind_group_layouts = &[
            &Screen::bind_group_layout(&self.device),
            &StorageBuffers::bind_group_layout(&self.device),
            &Atlas::bind_group_layout(&self.device),
            &Sampler::bind_group_layout(&self.device),
        ];

        let pipeline = Pipeline::new_render_pipeline(
            &self.device,
            self.config.format,
            buffers,
            bind_group_layouts
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(desc)],
            ..Default::default()
        });

        pass.set_pipeline(pipeline.get_render_pipeline());

        pass.set_index_buffer(self.mesh[self.current].indices_slice(), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, self.mesh[self.current].vertices_slice());

        pass.set_bind_group(0, &self.screen.bind_group, &[]);
        pass.set_bind_group(1, &self.storage[self.current].bind_group, &[]);
        pass.set_bind_group(2, &self.atlas.bind_group, &[]);
        pass.set_bind_group(3, &self.sampler.bind_group, &[]);

        pass.draw_indexed(0..self.mesh[self.current].offset as u32 * 6, 0, 0..1);
    }

    pub fn finish(&mut self) {
        let surface = self.target_texture.take().unwrap();
        let encoder = self.encoder.take().unwrap();
        let id = self.queue.submit([encoder.finish()]);
        let _ = self.device.poll(wgpu::PollType::WaitForSubmissionIndex(id));
        surface.present();
    }
}

pub struct Scene<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    storage: &'a mut StorageBuffers,
    mesh: &'a mut MeshBuffer,
    atlas: &'a mut Atlas,
    screen_res: Size,
    clear_color: &'a mut Rgba<f32>,
}

// FIXME: this feels immediate mode to me, idk
impl Scene<'_> {
    pub fn draw(
        self,
        transform: Matrix3x2,
        rotation: f32,
        background_paint: PaintRef<'_>,
        border_paint: PaintRef<'_>,
        border_width: f32,
        shape: Shape,
    ) {
        use aplite_storage::Entity;

        let offset = self.mesh.offset;

        let mut element = Element::new()
            .with_shape(shape)
            .with_rotation(rotation)
            .with_border_width(border_width);

        match border_paint {
            PaintRef::Color(rgba) => {
                element.border = rgba.f32();
            },
            PaintRef::Image(_image_ref) => {},
        }

        let atlas_id = match background_paint {
            PaintRef::Color(rgba) => {
                element.background = rgba.f32();
                None
            },
            PaintRef::Image(image_ref) => image_ref
                .upgrade()
                .and_then(|image| self.atlas.append(image)),
        };

        let indices = Indices::new().with_offset(offset as _, true);

        let vertices = atlas_id.and_then(|id| {
            element.set_atlas_id(id.index() as i32);

            self.atlas
                .get_uv(&id)
                .map(|uv| Vertices::new()
                    .with_uv(uv)
                    .with_id(offset as _)
                )
        })
        .unwrap_or(Vertices::new().with_id(offset as _));

        self.mesh
            .indices
            .write(self.device, self.queue, offset * 6, indices.as_slice());
        self.mesh
            .vertices
            .write(self.device, self.queue, offset * 4, vertices.as_slice());
        self.storage
            .elements
            .write(self.device, self.queue, offset, &[element]);
        self.storage
            .transforms
            .write(self.device, self.queue, offset, &[transform]);

        self.mesh.offset += 1;
    }

    pub fn size(&self) -> Size {
        self.screen_res
    }

    pub fn set_clear_color(&mut self, color: Rgba<f32>) {
        *self.clear_color = color;
    }
}

pub(crate) enum Pipeline {
    Render(wgpu::RenderPipeline),
    #[allow(unused)]
    // TODO: this is deep & complex topic, but nevertheless an interesting one to study
    Compute(wgpu::ComputePipeline),
}

impl Pipeline {
    pub(crate) fn new_render_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"), source: wgpu::ShaderSource::Wgsl(crate::shader::render())
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let blend_comp = wgpu::BlendComponent {
            operation: wgpu::BlendOperation::Add,
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: blend_comp,
                        alpha: blend_comp,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        Self::Render(pipeline)
    }

    pub(crate) fn get_render_pipeline(&self) -> &wgpu::RenderPipeline {
        match self {
            Pipeline::Render(render_pipeline) => render_pipeline,
            Pipeline::Compute(_) => panic!("expected render pipeline, get a compute instead"),
        }
    }
}

#[inline]
const fn backend() -> wgpu::Backends {
    #[cfg(all(unix, not(target_os = "macos")))]
    return wgpu::Backends::GL;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}
