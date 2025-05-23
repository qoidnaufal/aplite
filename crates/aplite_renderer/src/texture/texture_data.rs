use aplite_types::Size;

use crate::gpu::Gpu;

use super::ImageData;

#[derive(Debug)]
pub(crate) struct TextureData {
    texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl TextureData {
    pub(crate) fn new(gpu: &Gpu, image_data: ImageData) -> Self {
        let device = &gpu.device;
        let queue = &gpu.queue;

        let texture = Self::create_texture(device, image_data.size());
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Self::bind_group(device, &view);

        let ret = Self { texture, bind_group };
        ret.submit_texture(queue, &image_data);
        ret
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        })
    }

    pub(crate) fn bind_group(device: &wgpu::Device, view: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        })
    }

    // FIXME: integrate this, or better, create atlas & dynamic texture
    pub(crate) fn _update_texture(&mut self, gpu: &Gpu, image_data: &ImageData) {
        let size = image_data.size();
        if size.width() > self.texture.width() || size.height() > self.texture.height() {
            self.texture = Self::create_texture(&gpu.device, size);
        }
        self.submit_texture(&gpu.queue, image_data);
    }

    #[inline(always)]
    fn create_texture(device: &wgpu::Device, size: Size<u32>) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    fn submit_texture(&self, queue: &wgpu::Queue, image_data: &ImageData) {
        queue.write_texture(
            self.texture.as_image_copy(),
            image_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_data.width()),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: image_data.width(),
                height: image_data.height(),
                depth_or_array_layers: 1,
            }
        );
    }
}
