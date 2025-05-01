mod gpu;
mod buffer;
mod shader;
mod render_util;
mod texture;
mod element;

use util::Size;

use shader::SHADER;

pub(crate) use gpu::Gpu;
pub(crate) use texture::{TextureData, image_reader};
pub(crate) use element::{Element, Corners};
pub(crate) use render_util::{
    create_pipeline,
    IntoRenderComponent,
    IntoRenderSource,
    IntoTextureData,
};
pub(crate) use buffer::{
    Gfx,
    Screen,
    Buffer,
    Indices,
};

use crate::error::GuiError;
use crate::color::{Pixel, Rgba};

pub(crate) struct Renderer {
    pub(crate) gpu: Gpu,
    pub(crate) gfx: Gfx,
    pseudo_texture: TextureData,
    screen: Screen,
    pipeline: wgpu::RenderPipeline,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
}

impl Renderer {
    pub(crate) fn new(gpu: Gpu, mut gfx: Gfx) -> Self {
        gpu.configure();

        // this is important to avoid creating texture for every element
        let pseudo_texture = TextureData::new(&gpu, Pixel::from(Rgba::WHITE));
        let mut screen = Screen::new(&gpu.device, gpu.size());
        screen.write(&gpu.device, &gpu.queue);
        gfx.write(&gpu.device, &gpu.queue);

        let indices = gfx.indices(&gpu.device);
        let instances = gfx.instances(&gpu.device);
        let pipeline = create_pipeline(&gpu, &[Gfx::instance_desc()], &[
            &Screen::bind_group_layout(&gpu.device),
            &Gfx::bind_group_layout(&gpu.device),
            &TextureData::bind_group_layout(&gpu.device),
        ]);

        Self {
            gpu,
            gfx,
            screen,
            pipeline,
            indices,
            instances,
            pseudo_texture,
        }
    }

    pub(crate) fn resize(&mut self, new_size: Size<u32>) {
        let ps: Size<f32> = self.screen.initial_size().into();
        let ns: Size<f32> = new_size.into();
        let scale = ps / ns;

        if new_size.width > 0 && new_size.height > 0 {
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu.configure();
        }

        // update root's size
        self.gfx.transforms.update(0, |mat| {
            let s = ns / ps;
            let sw = 1.0 + s.width;
            let sh = 1.0 + s.height;
            mat.scale(sw, sh);
        });

        self.screen.update_transform(|mat| {
            mat.scale(scale.width, scale.height);
            mat.translate(scale.width - 1.0, 1.0 - scale.height);
        });
    }

    pub(crate) fn update(&mut self) {
        self.screen.write(&self.gpu.device, &self.gpu.queue);
        self.gfx.write(&self.gpu.device, &self.gpu.queue);
    }

    pub(crate) fn render(&mut self) -> Result<(), GuiError> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        self.encode(&mut encoder, &view);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn encode(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Rgba::BLACK.into()),
                    store: wgpu::StoreOp::Store,
                }
            })],
            ..Default::default()
        });

        if !self.gfx.is_empty() {
            pass.set_pipeline(&self.pipeline);
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.set_bind_group(0, &self.screen.bind_group, &[]);         // screen transform
            pass.set_bind_group(1, &self.gfx.bind_group, &[]);            // storage buffers
            pass.set_bind_group(2, &self.pseudo_texture.bind_group, &[]); // pseudo texture

            let mut idx_offset: u32 = 0;

            for i in 0..self.gfx.count() {
                let element = &self.gfx.elements.data[i];
                let idx_len = element.indices().len() as u32;
                let draw_offset = i as u32;

                // FIXME: bundle the texture into an atlas or something
                if element.texture_id > -1 {
                    let texture_data = &self.gfx.textures[element.texture_id as usize];
                    pass.set_bind_group(2, &texture_data.bind_group, &[]);
                }
                pass.draw_indexed(idx_offset..idx_offset + idx_len, 0, draw_offset..draw_offset + 1);
                idx_offset += idx_len;
            }
        }
    }
}
