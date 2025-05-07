use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

use image::GenericImageView;
use shared::Size;

use crate::color::Pixel;
use super::{Gpu, TextureDataSource};

pub(crate) fn image_reader<P: Into<PathBuf>>(path: P) -> Pixel<u8> {
    let mut file = File::open(path.into()).unwrap();
    let mut buf = Vec::new();
    let len = file.read_to_end(&mut buf).unwrap();
    let image = image::load_from_memory(&buf[..len]).unwrap();

    Pixel::new(image.dimensions(), &image.to_rgba8())
}

#[derive(Debug)]
pub(crate) struct TextureData {
    _texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl TextureData {
    pub(crate) fn new(gpu: &Gpu, td: &impl TextureDataSource) -> Self {
        let device = &gpu.device;
        let queue = &gpu.queue;

        let texture = Self::create_texture(device, td.dimensions());
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::create_sampler(device);
        let bind_group = Self::bind_group(device, &view, &sampler);

        Self::submit_texture(queue, texture.as_image_copy(), td);

        Self { _texture: texture, bind_group }
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
        })
    }

    pub(crate) fn bind_group(
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                }
            ],
        })
    }

    // FIXME: integrate this
    pub(crate) fn _update_texture(&mut self, gpu: &Gpu, td: &impl TextureDataSource) {
        let size = td.dimensions();
        if size.width() > self._texture.width() || size.height() > self._texture.height() {
            self._texture = Self::create_texture(&gpu.device, size);
        }
        Self::submit_texture(&gpu.queue, self._texture.as_image_copy(), td);
    }

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

    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    fn submit_texture(
        queue: &wgpu::Queue,
        texture: wgpu::TexelCopyTextureInfo,
        td: &impl TextureDataSource,
    ) {
        queue.write_texture(
            texture,
            td.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * td.dimensions().width()),
                rows_per_image: Some(td.dimensions().height()),
            },
            wgpu::Extent3d {
                width: td.dimensions().width(),
                height: td.dimensions().height(),
                depth_or_array_layers: 1,
            }
        );
    }
}
