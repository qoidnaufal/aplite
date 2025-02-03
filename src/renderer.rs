mod gpu;
mod buffer;
mod shader;
mod pipeline;
mod texture;

use std::collections::HashMap;
use buffer::Screen;
use math::Size;

pub use buffer::{Gfx, Buffer};
pub use pipeline::{bind_group_layout, bind_group, pipeline};
pub use gpu::Gpu;
pub use shader::SHADER;
pub use texture::{TextureData, image_reader};

use crate::shapes::Shape;
use crate::storage::{cast_slice, WidgetStorage};
use crate::error::Error;
use crate::NodeId;

pub struct Renderer<'a> {
    pub gpu: Gpu<'a>,
    pipeline: wgpu::RenderPipeline,
    screen: Screen,
    pub scenes: HashMap<NodeId, Gfx>,
}

impl<'a> Renderer<'a> {
    pub fn new(gpu: Gpu<'a>, widgets: &mut WidgetStorage) -> Self {
        gpu.configure();

        let screen = Screen::new(&gpu.device, gpu.size());
        let bg_layout = bind_group_layout(&gpu.device);
        let screen_bg_layout = Screen::bind_group_layout(&gpu.device);
        let pipeline = pipeline(&gpu.device, gpu.config.format, &[&bg_layout, &screen_bg_layout]);

        let mut scenes = HashMap::default();
        widgets.layout(gpu.size());
        widgets.prepare(&gpu.device, &gpu.queue, &bg_layout, &mut scenes);

        Self {
            gpu,
            pipeline,
            screen,
            scenes,
        }
    }

    pub fn resize(&mut self, size: Size<u32>) {
        let prev_size: Size<f32> = self.screen.initial_size.into();
        let new_size: Size<f32> = size.into();
        let s = prev_size / new_size;

        if size.width > 0 && size.height > 0 {
            self.gpu.config.width = size.width;
            self.gpu.config.height = size.height;
            self.gpu.configure();
        }

        self.screen.update(
            &self.gpu.device,
            &self.gpu.queue,
            |mat| {
                mat.scale(s.width, s.height);
                mat.translate(s.width - 1.0, 1.0 - s.height);
            }
        );
    }

    pub fn update(&mut self, id: &NodeId, shape: &Shape) {
        if let Some(gfx) = self.scenes.get_mut(id) {
            gfx.t.update_color(&self.gpu.queue, shape.color);
            gfx.u.update(&self.gpu.device, &self.gpu.queue, cast_slice(shape.transform.data()));
        }
    }

    pub fn render(&mut self, nodes: &[NodeId]) -> Result<(), Error> {
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
            nodes,
            &self.scenes,
            &self.screen,
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
    nodes: &[NodeId],
    scenes: &HashMap<NodeId, Gfx>,
    screen: &Screen,
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
    pass.set_bind_group(1, &screen.bind_group, &[]);
    for node_id in nodes {
        let gfx = &scenes[node_id];
        pass.set_bind_group(0, &gfx.bg, &[]);
        pass.set_index_buffer(gfx.i.slice(), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..gfx.i.count, 0, 0..1);
    }
}
