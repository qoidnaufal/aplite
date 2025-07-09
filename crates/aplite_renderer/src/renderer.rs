use std::sync::Arc;
use winit::window::Window;
use aplite_types::{Matrix3x2, Rgba, Size};

use super::RendererError;

use crate::element::Element;
use crate::screen::Screen;
use crate::storage::Storage;
use crate::gpu::Gpu;
use crate::mesh::{Indices, MeshBuffer, Vertex};
use crate::util::{create_pipeline, Sampler};
use crate::texture::{Atlas, ImageData, TextureData, TextureInfo};

pub struct Renderer {
    gpu: Gpu,
    screen: Screen,
    storage: [Storage; 3],
    atlas: Atlas,
    sampler: Sampler,
    images: Vec<TextureData>,
    pipeline: wgpu::RenderPipeline,
    mesh: [MeshBuffer; 3],
    current: usize,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let gpu = Gpu::new(Arc::clone(&window))?;

        let buffers = &[MeshBuffer::vertice_desc()];
        let layouts = &[
            &Screen::bind_group_layout(&gpu.device),
            &Storage::bind_group_layout(&gpu.device),
            &Atlas::bind_group_layout(&gpu.device),
            &Sampler::bind_group_layout(&gpu.device),
        ];
        let pipeline = create_pipeline(&gpu, buffers, layouts);

        let screen = Screen::new(&gpu.device, gpu.size().into(), window.scale_factor());
        let storage = [Storage::new(&gpu.device), Storage::new(&gpu.device), Storage::new(&gpu.device)];
        let atlas = Atlas::new(&gpu.device);
        let sampler = Sampler::new(&gpu.device);
        let mesh = [MeshBuffer::new(&gpu.device), MeshBuffer::new(&gpu.device), MeshBuffer::new(&gpu.device)];
        let images = vec![];

        Ok(Self {
            gpu,
            storage,
            sampler,
            atlas,
            images,
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

        self.screen.update_transform(|mat| {
            mat.set_scale(s.width(), s.height());
            mat.set_translate(s.width() - 1.0, 1.0 - s.height());
        });
    }

    pub fn render(&mut self, color: Rgba<u8>, window: Arc<Window>) -> Result<(), RendererError> {
        let frame = self.gpu.get_current_texture()?;
        let view = &frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        let desc = wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color.into()),
                store: wgpu::StoreOp::Store,
            }
        };

        self.atlas.update(&self.gpu.device, &mut encoder);
        self.encode(&mut encoder, desc);

        window.pre_present_notify();

        self.gpu.queue.submit([encoder.finish()]);
        frame.present();

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

impl Renderer {
    pub fn begin(&mut self) {
        self.current = (self.current + 1) % 3;
    }

    pub fn submit_data(
        &mut self,
        element: Element,
        transform: Matrix3x2,
        vertices: &[Vertex],
        offset: u64,
    ) {
        let indices = Indices::new().with_offset(offset as _, true).as_slice();
        self.mesh[self.current].indices.write(&self.gpu.device, &self.gpu.queue, offset * 6, &indices);
        self.mesh[self.current].vertices.write(&self.gpu.device, &self.gpu.queue, offset * 4, vertices);
        self.mesh[self.current].offset = offset;

        self.storage[self.current].elements.write(&self.gpu.device, &self.gpu.queue, offset, &[element]);
        self.storage[self.current].transforms.write(&self.gpu.device, &self.gpu.queue, offset, &[transform]);
    }

    pub fn finish(&mut self) {
        self.screen.write(&self.gpu.device, &self.gpu.queue);
    }

    pub fn submit_data_batched(
        &mut self,
        elements: &[Element],
        transforms: &[Matrix3x2],
        vertices: &[Vertex],
    ) {
        let indices = (0..elements.len())
            .flat_map(|i| Indices::new().with_offset(i as _, true).as_slice())
            .collect::<Vec<_>>();

        self.mesh[self.current].write_data(&self.gpu.device, &self.gpu.queue, &indices, vertices);
        self.storage[self.current].write_data(&self.gpu.device, &self.gpu.queue, elements, transforms);
    }

    pub fn push_image(&mut self, f: &dyn Fn() -> ImageData) -> TextureInfo {
        let image = f();
        let info = TextureInfo::ImageId(self.images.len() as _);
        let texture_data = TextureData::new(&self.gpu, image);
        self.images.push(texture_data);
        info
    }

    pub fn push_atlas(&mut self, f: &dyn Fn() -> ImageData) -> Option<TextureInfo> {
        let image = f();
        self.atlas.push(image)
    }
}
