use std::sync::Arc;
use winit::window::Window;

use crate::error::ApliteError;
use crate::image_data::ImageData;

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
use util::{create_pipeline, RenderComponentSource, Sampler, Model};
use shared::{Fraction, Matrix4x4, Rgba, Size};
use texture::TextureData;

pub(crate) struct Renderer {
    pub(crate) gpu: Gpu,
    pub(crate) gfx: Gfx,
    sampler: Sampler,
    textures: Vec<TextureData>,
    pseudo_texture: TextureData,
    pipeline: wgpu::RenderPipeline,
    pub(crate) model: Model,
    screen: Screen,
}

impl Renderer {
    pub(crate) fn new(window: Arc<Window>) -> Result<Self, ApliteError> {
        let gpu = Gpu::new(Arc::clone(&window))?;
        let gfx = Gfx::new(&gpu.device);
        gpu.configure();

        // FIXME: use atlas
        let pseudo_texture = TextureData::new(&gpu, &ImageData::from(Rgba::WHITE));
        let sampler = Sampler::new(&gpu.device);
        let screen = Screen::new(&gpu.device, gpu.size().into());

        let buffer_descriptors = &[Gfx::vertice_desc(), Gfx::instance_desc()];
        let bind_group_layouts = &[
            &Screen::bind_group_layout(&gpu.device),
            &Gfx::bind_group_layout(&gpu.device),
            &TextureData::bind_group_layout(&gpu.device),
            &Sampler::bind_group_layout(&gpu.device),
        ];
        let pipeline = create_pipeline(&gpu, buffer_descriptors, bind_group_layouts);
        let textures = vec![];
        let model = Model::Uninitialized;

        Ok(Self {
            gpu,
            gfx,
            sampler,
            textures,
            pseudo_texture,
            pipeline,
            model,
            screen,
        })
    }

    pub(crate) fn resize(&mut self, new_size: Size<u32>) {
        let ss = self.screen.initial_size();
        let ns: Size<f32> = new_size.into();
        let s = ss / ns;

        if new_size.width() > 0 && new_size.height() > 0 {
            self.gpu.config.width = new_size.width();
            self.gpu.config.height = new_size.height();
            self.gpu.configure();
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

    pub(crate) fn update(&mut self) {
        if self.model.is_unitialized() { self.model = Model::init(self) }
        self.screen.write(&self.gpu.queue);
        self.gfx.write(&self.gpu.device, &self.gpu.queue);
    }

    pub(crate) fn render(&mut self, color: Rgba<u8>) -> Result<(), ApliteError> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        self.encode(&mut encoder, &view, color);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

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

        if let Some((indices, vertices, instances)) = self.model.get_buffer() {
            pass.set_pipeline(&self.pipeline);
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_vertex_buffer(0, vertices.slice(..));
            pass.set_vertex_buffer(1, instances.slice(..));
            pass.set_bind_group(0, &self.screen.bind_group, &[]);         // screen transform
            pass.set_bind_group(1, &self.gfx.bind_group, &[]);            // storage buffers
            pass.set_bind_group(2, &self.pseudo_texture.bind_group, &[]); // pseudo texture
            pass.set_bind_group(3, &self.sampler.bind_group, &[]);

            let mut idx_start: u32 = 0;

            for i in 0..self.gfx.count() {
                let element = &self.gfx.elements.data[i];
                let draw_offset = i as u32;
                let idx_end = idx_start + 6;

                // FIXME: bundle the texture into an atlas or something
                if element.texture_id > -1 {
                    let texture_data = &self.textures[element.texture_id as usize];
                    pass.set_bind_group(2, &texture_data.bind_group, &[]);
                }
                pass.draw_indexed(idx_start..idx_end, 0, draw_offset..draw_offset + 1);
                idx_start = idx_end;
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct TextureInfo {
    pub(crate) id: i32,
    pub(crate) aspect_ratio: Fraction<u32>,
}

impl Renderer {
    pub(crate) fn add_texture(&mut self, f: &Box<dyn Fn() -> ImageData>) -> TextureInfo {
        let image = f();
        let aspect_ratio = image.aspect_ratio();
        let id = self.textures.len() as i32;
        let texture_data = TextureData::new(&self.gpu, &image);
        self.textures.push(texture_data);
        TextureInfo { id, aspect_ratio }
    }

    pub(crate) fn add_component(&mut self, rc: &impl RenderComponentSource) {
        let element = rc.element().with_transform_id(self.gfx.count() as u32);
        let transform = Matrix4x4::IDENTITY;

        self.gfx.elements.push(element);
        self.gfx.transforms.push(transform);
    }
}
