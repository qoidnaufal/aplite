use math::Size;

use crate::shapes::Transform;
use crate::texture::TextureCollection;
use crate::shapes::Vertex;
use crate::pipeline::Pipeline;
use crate::pipeline::bind_group_layout;
use crate::layout::Layout;
use crate::gpu::GpuResources;
use crate::error::Error;
use crate::buffer::Buffer;
use crate::app::CONTEXT;

pub struct GfxRenderer<'a> {
    pub gpu: GpuResources<'a>,
    pipeline: Pipeline,
    v_buffer: Buffer<Vertex>,
    i_buffer: Buffer<u32>,
    u_buffer: Buffer<Transform>,
    texture: TextureCollection,
}

impl<'a> GfxRenderer<'a> {
    pub fn new(gpu: GpuResources<'a>, layouts: &Layout) -> Self {
        let vertices = layouts.vertices();
        let indices = layouts.indices();
        let transforms = layouts.transforms();

        let bg_layout = bind_group_layout(&gpu.device);

        let v_buffer = Buffer::new(&gpu.device, wgpu::BufferUsages::VERTEX, vertices);
        let i_buffer = Buffer::new(&gpu.device, wgpu::BufferUsages::INDEX, indices);
        let u_buffer = Buffer::new(&gpu.device, wgpu::BufferUsages::UNIFORM, &transforms);

        let mut texture = TextureCollection::new(
            &gpu.device,
            &bg_layout,
            &u_buffer.buffer,
            Size::new(1, 1)
        );
        layouts.process_texture(&gpu.device, &gpu.queue, &bg_layout, &u_buffer.buffer, &mut texture);
        let pipeline = Pipeline::new(&gpu.device, gpu.config.format, &bg_layout);

        Self {
            gpu,
            pipeline,
            v_buffer,
            i_buffer,
            u_buffer,
            texture,
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
        // self.v_buffer.update(&self.gpu.queue, 0, data);
        self.u_buffer.update(&self.gpu.queue, 0, data);
    }

    pub fn render(&mut self, indices_len: usize) -> Result<(), Error> {
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
            &self.v_buffer,
            &self.i_buffer,
            indices_len,
            &self.texture.bind_group(),
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
    v_buffer: &Buffer<Vertex>,
    i_buffer: &Buffer<u32>,
    indices_len: usize,
    bind_group: &wgpu::BindGroup,
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
    pass.set_bind_group(0, bind_group, &[]);
    pass.set_vertex_buffer(0, v_buffer.slice());
    pass.set_index_buffer(i_buffer.slice(), wgpu::IndexFormat::Uint32);
    pass.draw_indexed(0..indices_len as u32, 0, 0..1);
}
