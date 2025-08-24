use std::sync::Weak;
use winit::window::Window;
use winit::dpi::PhysicalSize;
use aplite_types::{Rect, Matrix3x2, Size, PaintRef, CornerRadius};

// use super::RenderError;
use super::InitiationError;

use crate::atlas::{Atlas, Uv};
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
    bundle: Option<wgpu::RenderBundle>,

    // FIXME: not needed?
    screen: Screen,

    // FIXME: merge these two into Scene?
    storage: StorageBuffers,
    mesh: MeshBuffer,

    atlas: Atlas,
    sampler: Sampler,
    offset: u64,
}

impl Renderer {
    pub async fn new(window: Weak<Window>) -> Result<Self, InitiationError> {
        let window = window.upgrade().unwrap();
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

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

        let logical: winit::dpi::LogicalSize<f32> = size.to_logical(scale_factor);
        let screen_size = Size::new(logical.width, logical.height);

        let screen = Screen::new(&device, screen_size, scale_factor);
        let atlas = Atlas::new(&device, Size::new(2000., 2000.));
        let sampler = Sampler::new(&device);

        let storage = StorageBuffers::new(&device);

        let mesh = MeshBuffer::new(&device);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            bundle: None,
            storage,
            sampler,
            atlas,
            mesh,
            screen,
            offset: 0,
        })
    }

    #[inline(always)]
    pub const fn scale_factor(&self) -> f64 {
        self.screen.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.screen.scale_factor = scale_factor;
    }

    /// Corresponds to [`winit::dpi::LogicalSize<f32>`]
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

    pub fn begin(&mut self) {
        self.mesh.offset = 0;
    }

    #[inline(always)]
    pub fn scene(&mut self) -> Scene<'_> {
        Scene {
            size: &self.screen.screen_resolution,
            device: &self.device,
            queue: &self.queue,
            storage: &mut self.storage,
            mesh: &mut self.mesh,
            atlas: &mut self.atlas,
        }
    }

    pub fn finish(&mut self, window: &Window) {
        if self.mesh.offset == 0 { return }

        if self.bundle.is_none() || self.mesh.offset != self.offset {
            let bind_group_layouts = &[
                &Screen::bind_group_layout(&self.device),
                &StorageBuffers::bind_group_layout(&self.device),
                &Atlas::bind_group_layout(&self.device),
                &Sampler::bind_group_layout(&self.device),
            ];

            let pipeline = Pipeline::new_render_pipeline(
                &self.device,
                self.config.format,
                &[MeshBuffer::vertice_layout()],
                bind_group_layouts
            );

            let bundle_encoder = self.encode(&pipeline);
            let render_bundle = bundle_encoder.finish(&Default::default());
            self.bundle = Some(render_bundle);
        }

        let surface = self.surface.get_current_texture().unwrap();
        let view = surface.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let desc = wgpu::RenderPassColorAttachment {
            view: &view,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
            resolve_target: None,
            depth_slice: None,
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        self.atlas.update(&self.device, &mut encoder);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(desc)],
            ..Default::default()
        });

        if let Some(render_bundle) = self.bundle.as_ref() {
            pass.execute_bundles([render_bundle]);
        }

        self.offset = self.mesh.offset;
        drop(pass);

        window.pre_present_notify();

        self.queue.submit([encoder.finish()]);
        surface.present();
    }

    fn encode<'a>(&'a self, pipeline: &'a wgpu::RenderPipeline) -> wgpu::RenderBundleEncoder<'a> {
        let desc = wgpu::RenderBundleEncoderDescriptor {
            label: Some("bundle encoder"),
            color_formats: &[Some(self.config.format)],
            depth_stencil: None,
            sample_count: 1,
            multiview: None,
        };

        let mut encoder = self.device.create_render_bundle_encoder(&desc);

        encoder.set_pipeline(pipeline);

        encoder.set_index_buffer(self.mesh.indices_slice(), wgpu::IndexFormat::Uint32);
        encoder.set_vertex_buffer(0, self.mesh.vertices_slice());

        encoder.set_bind_group(0, &self.screen.bind_group, &[]);
        encoder.set_bind_group(1, &self.storage.bind_group, &[]);
        encoder.set_bind_group(2, &self.atlas.bind_group, &[]);
        encoder.set_bind_group(3, &self.sampler.bind_group, &[]);

        encoder.draw_indexed(0..self.mesh.offset as u32 * Indices::COUNT as u32, 0, 0..1);

        encoder
    }
}

pub struct Scene<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    storage: &'a mut StorageBuffers,
    mesh: &'a mut MeshBuffer,
    atlas: &'a mut Atlas,
    size: &'a Size,
}

pub struct DrawArgs<'a> {
    pub rect: &'a Rect,
    pub transform: &'a Matrix3x2,
    pub background_paint: &'a PaintRef<'a>,
    pub border_paint: &'a PaintRef<'a>,
    pub border_width: f32,
    pub shape: Shape,
    pub corner_radius: CornerRadius,
}

// FIXME: this feels immediate mode to me, idk
impl Scene<'_> {
    pub fn draw(
        &mut self,
        DrawArgs {
            rect,
            transform,
            background_paint,
            border_paint,
            border_width,
            shape,
            corner_radius,
        }: &DrawArgs<'_>,
    ) {
        let offset = self.mesh.offset;

        let mut element = Element::new(rect.size() / self.size)
            .with_shape(*shape)
            .with_corner_radius(corner_radius)
            .with_border_width(border_width / self.size.width);

        match border_paint {
            PaintRef::Color(rgba) => {
                element.border = rgba.pack_u32();
            },
            PaintRef::Image(_image_ref) => {
                todo!("not implemented yet")
            },
        }

        let vertices = match background_paint {
            PaintRef::Color(rgba) => {
                element.background = rgba.pack_u32();
                Vertices::new(
                    rect,
                    Uv {
                        min_x: 0.,
                        min_y: 0.,
                        max_x: 1.,
                        max_y: 1.,
                    },
                    self.size,
                    offset as _,
                    0,
                )
            },
            PaintRef::Image(image_ref) => {
                let uv = self.atlas.append(image_ref).unwrap();
                Vertices::new(&rect, uv, self.size, offset as _, 1)
            }
        };

        let indices = Indices::new(offset as _);

        self.mesh
            .indices
            .write(self.device, self.queue, offset * Indices::COUNT, indices.as_slice());
        self.mesh
            .vertices
            .write(self.device, self.queue, offset * Vertices::COUNT, vertices.as_slice());
        self.storage
            .elements
            .write(self.device, self.queue, offset, &[element]);
        self.storage
            .transforms
            .write(self.device, self.queue, offset, &[transform.as_slice()]);

        self.mesh.offset += 1;
    }

    pub fn next_frame(&mut self) {
        self.mesh.offset += 1;
    }

    pub fn size(&self) -> &Size {
        self.size
    }
}

// pub(crate) enum Pipeline {
//     Render(wgpu::RenderPipeline),
//     // TODO: this is deep & complex topic, but nevertheless an interesting one to study
//     Compute(wgpu::ComputePipeline),
// }

struct Pipeline;

impl Pipeline {
    pub(crate) fn new_render_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
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

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        })
    }

    // pub(crate) fn get_render_pipeline(&self) -> &wgpu::RenderPipeline {
    //     match self {
    //         Pipeline::Render(render_pipeline) => render_pipeline,
    //         Pipeline::Compute(_) => panic!("expected render pipeline, get a compute instead"),
    //     }
    // }
}

#[inline]
const fn backend() -> wgpu::Backends {
    if cfg!(target_os = "macos") {
        wgpu::Backends::METAL
    } else {
        wgpu::Backends::GL
    }
}

// pub struct Scene2<'a> {
//     indices: Vec<u32>,
//     vertices: Vec<crate::mesh::Vertex>,
//     elements: Vec<Element>,
//     transforms: Vec<Matrix3x2>,
//     size: Size,
//     atlas: &'a mut Atlas,
// }

// impl<'a> Scene2<'a> {
//     pub fn draw(
//         &mut self,
//         rect: &Rect,
//         transform: Matrix3x2,
//         background_paint: PaintRef<'_>,
//         border_paint: PaintRef<'_>,
//         border_width: f32,
//         shape: Shape,
//         corner_radius: &CornerRadius,
//     ) {
//         let mut element = Element::new(rect.size() / self.size)
//             .with_shape(shape)
//             .with_corner_radius(corner_radius)
//             .with_border_width(border_width / self.size.width);

//         match border_paint {
//             PaintRef::Color(rgba) => {
//                 element.border = rgba.pack_u32();
//             },
//             PaintRef::Image(_image_ref) => {
//                 todo!("not implemented yet")
//             },
//         }

//         let vertices = match background_paint {
//             PaintRef::Color(rgba) => {
//                 element.background = rgba.pack_u32();
//                 Vertices::new(
//                     rect,
//                     Uv {
//                         min_x: 0.,
//                         min_y: 0.,
//                         max_x: 1.,
//                         max_y: 1.,
//                     },
//                     self.size,
//                     self.elements.len() as _,
//                     0,
//                 )
//             },
//             PaintRef::Image(image_ref) => {
//                 let uv = self.atlas.append(image_ref).unwrap();
//                 Vertices::new(rect, uv, self.size, self.elements.len() as _, 1)
//             }
//         };

//         let indices = Indices::new(self.elements.len() as _);

//         self.indices.extend_from_slice(indices.as_slice());
//         self.vertices.extend_from_slice(vertices.as_slice());
//         self.transforms.push(transform);
//         self.elements.push(element);
//     }
// }
