use std::sync::Arc;
use winit::window::Window;
use aplite_types::{Matrix3x2, Rgba, Size};

use super::RendererError;

use crate::element::Element;
use crate::screen::Screen;
use crate::storage::Storage;
use crate::gpu::Gpu;
use crate::mesh::{Indices, MeshBuffer};
use crate::util::{create_pipeline, Sampler};
use crate::texture::{Atlas, AtlasId, ImageData};
use crate::Vertices;

pub struct Renderer {
    gpu: Gpu,
    screen: Screen,
    storage: [Storage; 3],
    atlas: Atlas,
    sampler: Sampler,
    pipeline: wgpu::RenderPipeline,
    mesh: [MeshBuffer; 3],
    current: usize,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let gpu = Gpu::new(Arc::clone(&window))?;

        let mut screen = Screen::new(&gpu.device, gpu.size().into(), window.scale_factor());
        let atlas = Atlas::new(&gpu.device);
        let sampler = Sampler::new(&gpu.device);
        let vertice_layout = &[MeshBuffer::vertice_layout()];

        let storage = [
            Storage::new(&gpu.device),
            Storage::new(&gpu.device),
            Storage::new(&gpu.device),
        ];
        let mesh = [
            MeshBuffer::new(&gpu.device),
            MeshBuffer::new(&gpu.device),
            MeshBuffer::new(&gpu.device),
        ];
        let layouts = &[
            &Screen::bind_group_layout(&gpu.device),
            &Storage::bind_group_layout(&gpu.device),
            &Atlas::bind_group_layout(&gpu.device),
            &Sampler::bind_group_layout(&gpu.device),
        ];

        let pipeline = create_pipeline(&gpu, vertice_layout, layouts);

        screen.size.write(&gpu.device, &gpu.queue, 0, &[gpu.size().into()]);

        Ok(Self {
            gpu,
            storage,
            sampler,
            atlas,
            pipeline,
            mesh,
            screen,
            current: 0,
        })
    }

    pub const fn scale_factor(&self) -> f64 { self.screen.scale_factor }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.screen.scale_factor = scale_factor;
    }

    /// Corresponds to [`winit::dpi::LogicalSize<u32>`]
    /// This one will be updated when the window is resized
    pub fn surface_size(&self) -> Size<u32> { self.gpu.size() }

    /// Corresponds to [`winit::dpi::LogicalSize<u32>`]
    /// This one will not be updated when the window is resized.
    /// Important to determine the transform of an [`Element`].
    pub fn screen_size(&self) -> Size<f32> { self.screen.screen_size() }

    pub fn resize(&mut self, new_size: Size<u32>) {
        let res = self.screen.screen_size();
        let ns = new_size.f32();
        let s = res / ns;

        if new_size.width() > 0 && new_size.height() > 0 {
            self.gpu.reconfigure_size(new_size);
        }

        let transform = Matrix3x2::IDENTITY
            .with_scale(s.width(), s.height())
            .with_translate(s.width() - 1.0, 1.0 - s.height());
        self.screen
            .transform
            .write(&self.gpu.device, &self.gpu.queue, 0, &[transform]);
        // self.screen.update_transform(|mat| {
        //     mat.set_scale(s.width(), s.height());
        //     mat.set_translate(s.width() - 1.0, 1.0 - s.height());
        // });
    }

    pub fn render(&mut self, color: Rgba<u8>, window: Arc<Window>) -> Result<(), RendererError> {
        let surface = self.gpu.get_surface_texture()?;
        let view = &surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.gpu.create_command_encoder();

        let desc = wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(
                    wgpu::Color {
                        r: color.r() as _,
                        g: color.g() as _,
                        b: color.b() as _,
                        a: color.a() as _,
                    }
                ),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        };

        self.atlas.update(&self.gpu.device, &mut encoder);
        self.encode(&mut encoder, desc);

        window.pre_present_notify();

        let submission_id = self.gpu.submit_encoder(encoder);
        self.gpu.poll_wait(submission_id)?;
        surface.present();

        Ok(())
    }

    fn encode(&self, encoder: &mut wgpu::CommandEncoder, desc: wgpu::RenderPassColorAttachment) {
        if self.mesh[self.current].offset == 0 { return }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(desc)],
            ..Default::default()
        });

        pass.set_pipeline(&self.pipeline);

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

    pub fn finish(&mut self) {
        // self.screen.write(&self.gpu.device, &self.gpu.queue);
    }

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
            .write(&self.gpu.device, &self.gpu.queue, offset * 6, indices.as_slice());
        self.mesh[self.current]
            .vertices
            .write(&self.gpu.device, &self.gpu.queue, offset * 4, vertices.as_slice());
        self.storage[self.current]
            .elements
            .write(&self.gpu.device, &self.gpu.queue, offset, &[element]);
        self.storage[self.current]
            .transforms
            .write(&self.gpu.device, &self.gpu.queue, offset, &[transform]);

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

        self.mesh[self.current].write_data(&self.gpu.device, &self.gpu.queue, &indices, &vertices);
        self.storage[self.current].write_data(&self.gpu.device, &self.gpu.queue, elements, transforms);
    }

    pub fn render_image(&mut self, f: &dyn Fn() -> ImageData) -> Option<AtlasId> {
        let image = f();
        self.atlas.append(image)
    }
}
