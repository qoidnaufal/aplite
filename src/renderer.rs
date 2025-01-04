use crate::{
    app::CONTEXT,
    buffer::Buffer,
    error::Error,
    gpu::GpuResources,
    layout::Layout,
    pipeline::Pipeline,
};

pub struct GfxRenderer<'a> {
    pub gpu: GpuResources<'a>,
    pipeline: Pipeline,
    buffer: Buffer,
}

impl<'a> GfxRenderer<'a> {
    pub fn new(gpu: GpuResources<'a>, layouts: &Layout) -> Self {
        let pipeline = Pipeline::new(&gpu.device, gpu.config.format);
        let vertices = layouts.vertices();
        let indices = layouts.indices();
        let buffer = Buffer::new(&gpu.device, &vertices, &indices);

        Self {
            gpu,
            pipeline,
            buffer,
        }
    }

    pub fn resize(&mut self) {
        let new_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu.configure();
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.buffer.update(&self.gpu.queue, data);
    }

    pub fn render(&mut self, indices_len: usize) -> Result<(), Error> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        draw(&mut encoder, &view, &self.pipeline, &self.buffer, indices_len);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn draw(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    pipeline: &Pipeline,
    buffer: &Buffer,
    indices_len: usize,
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
                    a: 1.,
                }),
                store: wgpu::StoreOp::Store,
            }
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_pipeline(&pipeline.pipeline);
    pass.set_vertex_buffer(0, buffer.v.slice(..));
    pass.set_index_buffer(buffer.i.slice(..), wgpu::IndexFormat::Uint32);
    pass.draw_indexed(0..indices_len as u32, 0, 0..1);
}
