use std::sync::Arc;
use winit::window::Window;
use aplite_types::{Matrix3x2, Rgba, Size};

use super::RendererError;

use crate::element::Element;
use crate::screen::Screen;
use crate::shader::render_shader;
use crate::storage::Storage;
use crate::mesh::{Indices, MeshBuffer};
use crate::util::Sampler;
use crate::texture::{Atlas, AtlasId, ImageData};
use crate::Vertices;

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    screen: Screen,
    storage: [Storage; 3],
    atlas: Atlas,
    sampler: Sampler,
    mesh: [MeshBuffer; 3],
    current: usize,
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, size: Size, scale_factor: f64) -> Self {
        let screen = Screen::new(&device, size, scale_factor);
        let atlas = Atlas::new(&device);
        let sampler = Sampler::new(&device);

        let storage = [
            Storage::new(&device),
            Storage::new(&device),
            Storage::new(&device),
        ];

        let mesh = [
            MeshBuffer::new(&device),
            MeshBuffer::new(&device),
            MeshBuffer::new(&device),
        ];

        Self {
            device,
            queue,
            storage,
            sampler,
            atlas,
            mesh,
            screen,
            current: 0,
        }
    }

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

    pub fn resize(&mut self, new_size: Size) {
        let res = self.screen_res();
        let ns = new_size;
        let scale = res / ns;
        let sx = scale.width;
        let sy = scale.height;

        self.screen
            .write(
                &self.device,
                &self.queue,
                Matrix3x2::IDENTITY
                    .with_scale(sx, sy)
                    .with_translate(sx - 1.0, 1.0 - sy),
                res
            );
    }

    pub(crate) fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        let label = Some("render encoder");
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label })
    }

    pub(crate) fn submit_encoder(&self, encoder: wgpu::CommandEncoder) -> wgpu::SubmissionIndex {
        self.queue.submit([encoder.finish()])
    }

    pub(crate) fn poll_wait(&self, index: wgpu::SubmissionIndex) -> Result<(), RendererError> {
        self.device.poll(wgpu::PollType::WaitForSubmissionIndex(index))?;
        Ok(())
    }

    pub fn render(
        &mut self,
        color: Rgba<u8>,
        window: Arc<Window>,
        surface: wgpu::SurfaceTexture,
        format: wgpu::TextureFormat,
    ) -> Result<(), RendererError> {
        let view = &surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.create_command_encoder();

        let desc = wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(
                    wgpu::Color {
                        r: color.r as _,
                        g: color.g as _,
                        b: color.b as _,
                        a: color.a as _,
                    }
                ),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        };

        self.atlas.update(&self.device, &mut encoder);
        self.encode(&mut encoder, desc, format);

        window.pre_present_notify();

        let submission_id = self.submit_encoder(encoder);
        self.poll_wait(submission_id)?;
        surface.present();

        Ok(())
    }

    fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        desc: wgpu::RenderPassColorAttachment,
        format: wgpu::TextureFormat,
    ) {
        if self.mesh[self.current].offset == 0 { return }

        let buffers = &[MeshBuffer::vertice_layout()];
        let bind_group_layouts = &[
            &Screen::bind_group_layout(&self.device),
            &Storage::bind_group_layout(&self.device),
            &Atlas::bind_group_layout(&self.device),
            &Sampler::bind_group_layout(&self.device),
        ];

        let pipeline = Pipeline::new_render_pipeline(
            &self.device,
            format,
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
}

// FIXME: this feels immediate mode to me, idk
impl Renderer {
    pub fn begin(&mut self) {
        self.current = (self.current + 1) % 3;
    }

    // pub fn finish(&mut self) {
    // }

    pub fn submit_data(
        &mut self,
        element: Element,
        transform: Matrix3x2,
        offset: u64,
    ) {
        let indices = Indices::new().with_offset(offset as _, true);
        let vertices = self.atlas
            .get_uv(&element.atlas_id)
            .map(|uv| Vertices::new().with_uv(*uv).with_id(offset as _))
            .unwrap_or(Vertices::new().with_id(offset as _));

        self.mesh[self.current]
            .indices
            .write(&self.device, &self.queue, offset * 6, indices.as_slice());
        self.mesh[self.current]
            .vertices
            .write(&self.device, &self.queue, offset * 4, vertices.as_slice());
        self.storage[self.current]
            .elements
            .write(&self.device, &self.queue, offset, &[element]);
        self.storage[self.current]
            .transforms
            .write(&self.device, &self.queue, offset, &[transform]);

        self.mesh[self.current].offset = offset + 1;
    }

    pub fn submit_data_batched(
        &mut self,
        elements: &[Element],
        transforms: &[Matrix3x2],
    ) {
        let mut indices = vec![];
        let mut vertices = vec![];
        (0..elements.len())
            .for_each(|i| {
                let atlas_id = elements[i].atlas_id();
                let idx = Indices::new().with_offset(i as _, true);
                let vert = self.atlas
                    .get_uv(&atlas_id)
                    .map(|uv| Vertices::new().with_uv(*uv).with_id(i as _))
                    .unwrap_or(Vertices::new().with_id(i as _));
                indices.extend_from_slice(idx.as_slice());
                vertices.extend_from_slice(&vert);
            });

        self.mesh[self.current].write_data(&self.device, &self.queue, &indices, &vertices);
        self.storage[self.current].write_data(&self.device, &self.queue, elements, transforms);
    }

    pub fn render_image(&mut self, f: &dyn Fn() -> ImageData) -> Option<AtlasId> {
        let image = f();
        self.atlas.append(image)
    }
}

#[allow(unused)]
pub(crate) enum Pipeline {
    Render(wgpu::RenderPipeline),
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
            label: Some("shader"), source: wgpu::ShaderSource::Wgsl(render_shader())
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

pub struct Scene {
    surface_size: Size,
    triangles: Vec<Vertices>,
}
