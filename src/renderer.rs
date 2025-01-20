use crate::buffer::Gfx;
use crate::pipeline::Pipeline;
use crate::pipeline::bind_group_layout;
use crate::widget_tree::WidgetTree;
use crate::gpu::GpuResources;
use crate::error::Error;
use crate::app::CONTEXT;
use crate::NodeId;

pub struct Renderer<'a> {
    pub gpu: GpuResources<'a>,
    pipeline: Pipeline,
    pub gfx: Gfx
}

impl<'a> Renderer<'a> {
    pub fn new(gpu: GpuResources<'a>, widgets: &WidgetTree) -> Self {
        let bg_layout = bind_group_layout(&gpu.device);
        let mut gfx = Gfx::default();
        widgets.process_texture(&gpu.device, &gpu.queue, &bg_layout, &mut gfx);

        let pipeline = Pipeline::new(&gpu.device, gpu.config.format, &bg_layout);

        Self {
            gpu,
            pipeline,
            gfx,
        }
    }

    pub fn resize(&mut self, widgets: &mut WidgetTree) {
        let nws = CONTEXT.with_borrow(|ctx| ctx.window_size);
        if nws.width > 0 && nws.height > 0 {
            self.gpu.config.width = nws.width;
            self.gpu.config.height = nws.height;
            self.gpu.configure();
        }
        widgets.recalculate_layout(&self.gpu.queue, &self.gfx);
    }

    pub fn update(&mut self, data: &[u8], id: &NodeId) {
        if let Some(texture) = self.gfx.textures.iter().find(|t| t.node_id == *id) {
            texture.u_buffer.update(&self.gpu.queue, 0, data);
        }
    }

    pub fn render(&mut self) -> Result<(), Error> {
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
    gfx: &Gfx,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            }
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_pipeline(&pipeline);
    for texture in &gfx.textures {
        let v = &gfx.v_buffer[texture.node_id.0 as usize];
        let i = &gfx.i_buffer[texture.node_id.0 as usize];

        pass.set_bind_group(0, &texture.bind_group, &[]);
        pass.set_vertex_buffer(0, v.slice());
        pass.set_index_buffer(i.slice(), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..i.len, 0, 0..1);
    }
}
