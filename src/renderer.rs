use std::sync::Arc;
use winit::window::Window;

use crate::error::ApliteError;

pub(crate) mod gpu;
pub(crate) mod buffer;
pub(crate) mod shader;
pub(crate) mod util;
pub(crate) mod texture;
pub(crate) mod element;
pub(crate) mod gfx;
mod screen;

use screen::Screen;
use gfx::Gfx;
use gpu::Gpu;
use util::{create_pipeline, RenderComponentSource, Sampler};
use shared::{Fraction, Matrix4x4, Rect, Rgba, Size};
use texture::{ImageData, TextureData};
use texture::atlas::Atlas;
use buffer::MeshBuffer;

pub(crate) struct Renderer {
    gpu: Gpu,
    gfx: Gfx,
    sampler: Sampler,
    atlas: Atlas,
    images: Vec<TextureData>,
    pipeline: wgpu::RenderPipeline,
    mesh: MeshBuffer,
    screen: Screen,
}

impl Renderer {
    pub(crate) fn new(window: Arc<Window>) -> Result<Self, ApliteError> {
        let gpu = Gpu::new(Arc::clone(&window))?;
        let gfx = Gfx::new(&gpu.device);
        gpu.configure();

        let sampler = Sampler::new(&gpu.device);
        let screen = Screen::new(&gpu.device, gpu.size().into(), window.scale_factor());

        let buffers = &[Gfx::vertice_desc(), Gfx::instance_desc()];
        let layouts = &[
            &Screen::bind_group_layout(&gpu.device),
            &Gfx::bind_group_layout(&gpu.device),
            &Atlas::bind_group_layout(&gpu.device),
            &Sampler::bind_group_layout(&gpu.device),
        ];
        let pipeline = create_pipeline(&gpu, buffers, layouts);
        let atlas = Atlas::new(&gpu.device);
        let mesh = MeshBuffer::Uninitialized;
        let images = vec![];

        Ok(Self {
            gpu,
            gfx,
            sampler,
            atlas,
            images,
            pipeline,
            mesh,
            screen,
        })
    }

    pub(crate) const fn scale_factor(&self) -> f64 { self.screen.scale_factor }

    pub(crate) fn set_scale_factor(&mut self, scale_factor: f64) {
        self.screen.scale_factor = scale_factor;
    }

    /// this one corresponds to [`winit::dpi::LogicalSize<u32>`]
    pub(crate) fn surface_size(&self) -> Size<u32> { self.gpu.size() }

    pub(crate) fn resize(&mut self, new_size: Size<u32>) {
        let ss = self.screen.initial_size();
        let ns: Size<f32> = new_size.into();
        let s = ss / ns;

        if new_size.width() > 0 && new_size.height() > 0 {
            self.gpu.reconfigure_size(new_size);
        }

        // FIXME: is it necessary to have a dummy root widget?
        // self.gfx.transforms.update(0, |mat| {
        //     let s = ns / ps;
        //     let sw = 1.0 + s.width;
        //     let sh = 1.0 + s.height;
        //     mat.scale(sw, sh);
        // });

        self.screen.update_transform(|mat| {
            mat.set_scale(s.width(), s.height());
            mat.set_translate(s.width() - 1.0, 1.0 - s.height());
        });
    }

    pub(crate) fn write_data(&mut self) {
        self.screen.write(&self.gpu.queue);
        let realloc = self.gfx.write(&self.gpu.device, &self.gpu.queue);
        if self.mesh.is_uninit() || realloc { self.mesh.init(&self.gfx, &self.gpu.device) }
    }

    pub(crate) fn submit_buffer(&mut self, color: Rgba<u8>) -> Result<(), wgpu::SurfaceError> {
        let output = self.gpu.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        self.atlas.update(&self.gpu.device, &mut encoder);
        self.encode(&mut encoder, &view, color);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[inline(always)]
    fn encode(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, color: Rgba<u8>) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color.into()),
                    store: wgpu::StoreOp::Store,
                }
            })],
            ..Default::default()
        });

        if let Some((idx, vtx, itc)) = self.mesh.get_buffer() {
            pass.set_pipeline(&self.pipeline);
            pass.set_index_buffer(idx.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_vertex_buffer(0, vtx.slice(..));
            pass.set_vertex_buffer(1, itc.slice(..));
            pass.set_bind_group(0, &self.screen.bind_group, &[]);
            pass.set_bind_group(1, &self.gfx.bind_group, &[]);
            pass.set_bind_group(2, &self.atlas.bind_group, &[]);
            pass.set_bind_group(3, &self.sampler.bind_group, &[]);

            let mut start: u32 = 0;

            for i in 0..self.gfx.count() {
                let element = &self.gfx.elements.data[i];
                let draw_offset = i as u32;
                let end = start + 6;

                if element.texture_id > -1 {
                    let image_bind_group = &self.images[element.texture_id as usize].bind_group;
                    pass.set_bind_group(2, image_bind_group, &[]);
                }
                pass.draw_indexed(start..end, 0, draw_offset..draw_offset + 1);
                start = end;
            }
        }
    }
}

impl Renderer {
    pub(crate) fn update_element_color(&mut self, index: usize, color: Rgba<u8>) {
        self.gfx.elements.update(index, |elem| elem.set_color(color));
    }

    pub(crate) fn update_element_size(&mut self, index: usize, size: Size<u32>) {
        self.gfx.elements.update(index, |elem| elem.set_size(size));
    }

    pub(crate) fn update_element_transform(&mut self, index: usize, rect: Rect<u32>) {
        let scaler = self.screen.initial_size();
        let size: Size<f32> = rect.size().into();
        self.gfx.transforms.update(index, |matrix| {
            let x = rect.x() as f32 / scaler.width() * 2.0 - 1.0;
            let y = 1.0 - rect.y() as f32 / scaler.height() * 2.0;
            let s = size / scaler;
            matrix.set_translate(x, y);
            matrix.set_scale(s.width(), s.height());
        });
    }
}

pub(crate) struct ImageInfo {
    pub(crate) id: i32,
    pub(crate) aspect_ratio: Fraction<u32>,
}

impl Renderer {
    pub(crate) fn push_image(&mut self, f: &Box<dyn Fn() -> ImageData>) -> ImageInfo {
        let image = f();
        let aspect_ratio = image.aspect_ratio();
        let id = self.images.len() as i32;
        let texture_data = TextureData::new(&self.gpu, &image);
        self.images.push(texture_data);
        ImageInfo { id, aspect_ratio }
    }

    pub(crate) fn add_component(&mut self, rc: &impl RenderComponentSource) {
        let element = rc.element().with_transform_id(self.gfx.count() as u32);
        let transform = Matrix4x4::IDENTITY;

        self.gfx.elements.push(element);
        self.gfx.transforms.push(transform);
    }
}
