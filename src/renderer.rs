mod gpu;
mod buffer;
mod shader;
mod pipeline;

pub use buffer::{Gfx, Buffer};
pub use pipeline::Pipeline;
pub use pipeline::{bind_group_layout, bind_group};
pub use gpu::Gpu;
pub use shader::SHADER;

use crate::context::CONTEXT;
use crate::shapes::Shape;
use crate::storage::WidgetStorage;
use crate::error::Error;
use crate::{NodeId, Rgb};

pub struct Renderer<'a> {
    pub gpu: Gpu<'a>,
    pipeline: Pipeline,
    pub gfx: Gfx
}

impl<'a> Renderer<'a> {
    pub fn new(gpu: Gpu<'a>, widgets: &WidgetStorage) -> Self {
        let bg_layout = bind_group_layout(&gpu.device);
        let mut gfx = Gfx::default();
        let pipeline = Pipeline::new(&gpu.device, gpu.config.format, &bg_layout);
        widgets.prepare(&gpu.device, &gpu.queue, &bg_layout, &mut gfx);

        Self {
            gpu,
            pipeline,
            gfx,
        }
    }

    pub fn resize(&mut self, widgets: &mut WidgetStorage) {
        let nws = CONTEXT.with_borrow(|ctx| ctx.window_size);
        if nws.width > 0 && nws.height > 0 {
            self.gpu.config.width = nws.width;
            self.gpu.config.height = nws.height;
            self.gpu.configure();
        }
        widgets.layout();
    }

    pub fn update(&mut self, id: &NodeId, shape: &Shape) {
        if let Some(texture) = self.gfx.textures.get_mut(id) {
            let data = shape.transform.as_slice();
            texture.change_color(&self.gpu.queue, shape.color);
            texture.u_buffer.update(&self.gpu.device, &self.gpu.queue, 0, data);
        }
    }

    pub fn render(&mut self, nodes: &[NodeId]) -> Result<(), Error> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        draw(
            &mut encoder,
            &view,
            &self.pipeline.pipeline,
            nodes,
            &self.gfx,
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn draw(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    nodes: &[NodeId],
    gfx: &Gfx,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(Rgb::DARK_GRAY.into()),
                store: wgpu::StoreOp::Store,
            }
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_pipeline(pipeline);
    for node_id in nodes {
        let v = &gfx.v_buffer[node_id];
        let i = &gfx.i_buffer[node_id];
        let t = &gfx.textures[node_id];

        pass.set_bind_group(0, &t.bind_group, &[]);
        pass.set_vertex_buffer(0, v.slice());
        pass.set_index_buffer(i.slice(), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..i.materials, 0, 0..1);
    }
}
