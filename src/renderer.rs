mod gpu;
mod buffer;
mod shader;
mod pipeline;
mod texture;

use std::sync::Arc;
use winit::window::Window;
use util::Size;

pub use buffer::{Gfx, Uniform};
pub use pipeline::pipeline;
pub use gpu::Gpu;
pub use shader::SHADER;
pub use texture::{TextureData, image_reader};

use crate::storage::WidgetStorage;
use crate::error::Error;
use crate::{IntoView, View};

pub struct Renderer {
    pub gpu: Gpu,
    pub gfx: Gfx,
    pipeline: wgpu::RenderPipeline,
    uniform: Uniform,
    textures: Vec<TextureData>,
    indices: wgpu::Buffer,
}

impl Renderer {
    pub fn new<F, IV>(
        window: Arc<Window>,
        storage: &mut WidgetStorage,
        view_fn: F
    ) -> Self
    where
        F: Fn() -> IV + 'static,
        IV: IntoView + 'static,
    {
        let gpu = Gpu::request(window).unwrap();
        gpu.configure();

        let mut gfx = Gfx::new(&gpu.device);
        let mut textures = Vec::new();
        let uniform = Uniform::new(&gpu.device, gpu.size());
        let pipeline = pipeline(&gpu, &[
            &Uniform::bind_group_layout(&gpu.device),
            &Gfx::bind_group_layout(&gpu.device),
            &TextureData::bind_group_layout(&gpu.device),
        ]);

        view_fn().into_view().prepare(storage, &gpu, &mut gfx, &mut textures);
        gfx.write(&gpu.device, &gpu.queue);
        let indices = gfx.indices(&gpu.device);

        Self {
            gpu,
            gfx,
            pipeline,
            uniform,
            textures,
            indices
        }
    }

    pub fn resize(&mut self, size: Size<u32>) {
        let prev_size: Size<f32> = self.uniform.initial_size.into();
        let new_size: Size<f32> = size.into();
        let s = prev_size / new_size;

        if size.width > 0 && size.height > 0 {
            self.gpu.config.width = size.width;
            self.gpu.config.height = size.height;
            self.gpu.configure();
        }

        self.uniform.update(|mat| {
            mat.scale(s.width, s.height);
            mat.translate(s.width - 1.0, 1.0 - s.height);
        });
        self.uniform.write(&self.gpu.device, &self.gpu.queue);
    }

    pub fn update(&mut self, index: usize) {
        self.gfx.write(&self.gpu.device, &self.gpu.queue);
        if let Some(texture_data) = self.textures.get(index) {
            let shape = &self.gfx.shapes.data[index];
            texture_data.update_color(&self.gpu.queue, shape.color.into());
        }
    }

    pub fn render(&mut self) -> Result<(), Error> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        encode(
            &mut encoder,
            &view,
            &self.pipeline,
            &self.gfx,
            &self.uniform,
            &self.textures,
            &self.indices,
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn encode(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    gfx: &Gfx,
    uniform: &Uniform,
    textures: &Vec<TextureData>,
    indices: &wgpu::Buffer,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            }
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &uniform.bind_group, &[]);
    pass.set_bind_group(1, &gfx.bind_group, &[]);
    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);

    let instance_len = gfx.shapes.len();
    let mut idx_offset: u32 = 0;
    let mut instace_offset: u32 = 0; 

    for i in 0..instance_len {
        let texture_data = &textures[i];
        let shape = &gfx.shapes.data[i];
        let idx_len = shape.indices().len() as u32;

        // FIXME: texture handling using storage / array
        pass.set_bind_group(2, &texture_data.bind_group, &[]);
        pass.draw_indexed(idx_offset..idx_offset + idx_len, 0, instace_offset..instace_offset + 1);

        idx_offset += idx_len;
        instace_offset += 1;
    }
}
