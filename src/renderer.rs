mod gpu;
mod buffer;
mod shader;
mod pipeline;

use std::collections::HashMap;

pub use buffer::{Gfx, Buffer};
use math::Size;
pub use pipeline::{bind_group_layout, bind_group, pipeline};
pub use gpu::Gpu;
pub use shader::SHADER;

use crate::shapes::Shape;
use crate::storage::WidgetStorage;
use crate::error::Error;
use crate::NodeId;

pub struct Renderer<'a> {
    pub gpu: Gpu<'a>,
    pipeline: wgpu::RenderPipeline,
    pub scenes: HashMap<NodeId, Gfx>,
}

impl<'a> Renderer<'a> {
    pub fn new(gpu: Gpu<'a>, widgets: &mut WidgetStorage) -> Self {
        let bg_layout = bind_group_layout(&gpu.device);
        let pipeline = pipeline(&gpu.device, gpu.config.format, &bg_layout);
        let mut scenes = HashMap::default();
        gpu.configure();
        widgets.layout(gpu.size());
        widgets.prepare(&gpu.device, &gpu.queue, &bg_layout, &mut scenes);

        Self {
            gpu,
            pipeline,
            scenes,
        }
    }

    pub fn resize(&mut self, widgets: &mut WidgetStorage, size: Size<u32>) {
        if size.width > 0 && size.height > 0 {
            self.gpu.config.width = size.width;
            self.gpu.config.height = size.height;
            self.gpu.configure();
        }
        widgets.layout(self.gpu.size());
    }

    pub fn update(&mut self, id: &NodeId, shape: &Shape) {
        if let Some(gfx) = self.scenes.get_mut(id) {
            let data = shape.transform.as_slice();
            gfx.t.update_color(&self.gpu.queue, shape.color);
            gfx.u.update(&self.gpu.device, &self.gpu.queue, 0, data);
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
    for node_id in nodes {
        let gfx = &scenes[node_id];
        pass.set_bind_group(0, &gfx.bg, &[]);
        pass.set_index_buffer(gfx.i.slice(), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..gfx.i.count, 0, 0..1);
    }
}
