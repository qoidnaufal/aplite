use std::sync::Arc;
use winit::window::Window;
use aplite_types::{Matrix3x2, Rect, Rgba, Size};

use super::RendererError;

use crate::element::Element;
use crate::screen::Screen;
use crate::storage::Storage;
use crate::gpu::Gpu;
use crate::mesh::MeshBuffer;
use crate::util::{create_pipeline, RenderElementSource, Sampler};
use crate::texture::{Atlas, ImageData, TextureData, TextureInfo};

pub struct Renderer {
    gpu: Gpu,
    screen: Screen,
    storage: Storage, // FIXME: change this into vertex buffer to enable batching
    atlas: Atlas,
    sampler: Sampler,
    images: Vec<TextureData>,
    pipeline: wgpu::RenderPipeline,
    mesh: MeshBuffer,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let gpu = Gpu::new(Arc::clone(&window))?;

        let buffers = &[MeshBuffer::vertice_desc()];
        let layouts = &[
            &Screen::bind_group_layout(&gpu.device),
            &Storage::bind_group_layout(&gpu.device),
            &Atlas::bind_group_layout(&gpu.device),
            &Sampler::bind_group_layout(&gpu.device),
        ];
        let pipeline = create_pipeline(&gpu, buffers, layouts);

        let screen = Screen::new(&gpu.device, gpu.size().into(), window.scale_factor());
        let storage = Storage::new(&gpu.device);
        let atlas = Atlas::new(&gpu.device);
        let sampler = Sampler::new(&gpu.device);
        let mesh = MeshBuffer::new(&gpu.device);
        let images = vec![];

        Ok(Self {
            gpu,
            storage,
            sampler,
            atlas,
            images,
            pipeline,
            mesh,
            screen,
        })
    }

    pub const fn scale_factor(&self) -> f64 { self.screen.scale_factor }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.screen.scale_factor = scale_factor;
    }

    /// this one corresponds to [`winit::dpi::LogicalSize<u32>`]
    pub fn surface_size(&self) -> Size<u32> { self.gpu.size() }

    pub fn resize(&mut self, new_size: Size<u32>) {
        let res = self.screen.screen_size();
        let ns: Size<f32> = new_size.into();
        let s = res / ns;

        if new_size.width() > 0 && new_size.height() > 0 {
            self.gpu.reconfigure_size(new_size);
        }

        self.screen.update_transform(|mat| {
            mat.set_scale(s.width(), s.height());
            mat.set_translate(s.width() - 1.0, 1.0 - s.height());
        });
    }

    pub fn write_data(&mut self) {
        self.screen.write(&self.gpu.device, &self.gpu.queue);
        self.storage.write(&self.gpu.device, &self.gpu.queue);
        self.mesh.write(&self.gpu.device, &self.gpu.queue);
    }

    pub fn render<P: FnOnce()>(
        &mut self,
        color: Rgba<u8>,
        pre_present_notify: P
    ) -> Result<(), RendererError> {
        let frame = self.gpu.get_current_texture()?;
        let view = &frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        let desc = wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color.into()),
                store: wgpu::StoreOp::Store,
            }
        };

        self.atlas.update(&self.gpu.device, &mut encoder);
        self.encode(&mut encoder, desc);

        pre_present_notify();

        self.gpu.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }

    fn encode(&self, encoder: &mut wgpu::CommandEncoder, desc: wgpu::RenderPassColorAttachment) {
        if self.mesh.offset == 0 { return }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(desc)],
            ..Default::default()
        });

        pass.set_pipeline(&self.pipeline);

        pass.set_index_buffer(self.mesh.indices.slice(0..self.mesh.offset * 6), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, self.mesh.vertices.slice(0..self.mesh.offset * 4));

        pass.set_bind_group(0, &self.screen.bind_group, &[]);
        pass.set_bind_group(1, &self.storage.bind_group, &[]);
        pass.set_bind_group(2, &self.atlas.bind_group, &[]);
        pass.set_bind_group(3, &self.sampler.bind_group, &[]);

        pass.draw_indexed(0..self.mesh.offset as u32 * 6, 0, 0..1);

        // TODO: organize how to render "non-atlased" image
        // self
        //     .storage
        //     .element_data
        //     .iter()
        //     .enumerate()
        //     .filter_map(|(idx, element)| {
        //         if element.image_id > -1 {
        //             Some((idx as u64, element.image_id as usize))
        //         } else { None }
        //     })
        //     .for_each(|(idx, image_id)| {
        //         let bind_group = &self.images[image_id].bind_group;
        //         pass.set_index_buffer(self.mesh.indices.slice(idx + 6..idx * 6), wgpu::IndexFormat::Uint32);
        //         pass.set_vertex_buffer(0, self.mesh.vertices.slice(idx + 4..idx * 4));
        //         pass.set_bind_group(2, bind_group, &[]);
        //         pass.draw_indexed(0..4, 0, 0..1);
        //     });
    }
}

impl Renderer {
    pub fn update_element_color(&mut self, index: usize, color: Rgba<u8>) {
        self.storage.update_element(index, |elem| elem.set_color(color));
    }

    pub fn update_element_size(&mut self, index: usize, size: Size<u32>) {
        self.storage.update_element(index, |elem| elem.set_size(size));
    }

    pub fn update_element_transform(&mut self, index: usize, rect: Rect<u32>) {
        let res = self.screen.screen_size();
        let size: Size<f32> = rect.size().into();
        self.storage.update_transform(index, |matrix| {
            let x = rect.x() as f32 / res.width() * 2.0 - 1.0;
            let y = 1.0 - rect.y() as f32 / res.height() * 2.0;
            let s = size / res;
            matrix.set_translate(x, y);
            matrix.set_scale(s.width(), s.height());
        });
    }
}

impl Renderer {
    pub fn push_image(&mut self, f: &dyn Fn() -> ImageData) -> TextureInfo {
        let image = f();
        let info = TextureInfo::ImageId(self.images.len() as _);
        let texture_data = TextureData::new(&self.gpu, image);
        self.images.push(texture_data);
        info
    }

    pub fn push_atlas(&mut self, f: &dyn Fn() -> ImageData) -> Option<TextureInfo> {
        let image = f();
        self.atlas.push(image)
    }

    pub fn add_component(&mut self,
        rcs: &impl RenderElementSource,
        texture_info: Option<TextureInfo>,
    ) {
        let (image_id, atlas_id, uv) = texture_info.map(|i| {
            let image_id = i.get_image_id().unwrap_or(-1);
            let atlas_id = i.get_atlas_id().unwrap_or(-1);
            let uv = i.get_uv().unwrap_or(Rect::new((0.0, 0.0), (1.0, 1.0)));
            (image_id, atlas_id, uv)
        }).unwrap_or((-1, -1, Rect::new((0.0, 0.0), (1.0, 1.0))));

        let element = Element::new(rcs)
            .with_transform_id(self.storage.count() as u32)
            .with_image_id(image_id)
            .with_atlas_id(atlas_id);

        self.storage.element_data.push(element);
        self.storage.transform_data.push(Matrix3x2::IDENTITY);
        self.mesh.uvs.push(uv);
    }
}
