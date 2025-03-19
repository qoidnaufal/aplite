use std::path::PathBuf;
use std::io::Read;
use std::fs::File;

use image::GenericImageView;

use crate::color::Pixel;
use crate::Rgba;

use super::Gpu;

pub fn image_reader<P: Into<PathBuf>>(path: P) -> Pixel<Rgba<u8>> {
    let mut file = File::open(path.into()).unwrap();
    let mut buf = Vec::new();
    let len = file.read_to_end(&mut buf).unwrap();
    let image = image::load_from_memory(&buf[..len]).unwrap();

    Pixel::new(image.dimensions(), &image.to_rgba8())
}

#[derive(Debug)]
pub struct TextureData {
    texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl TextureData {
    pub fn new(gpu: &Gpu, pixel: &Pixel<Rgba<u8>>) -> Self {
        let device = &gpu.device;
        let queue = &gpu.queue;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: pixel.dimensions().width,
                height: pixel.dimensions().height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = sampler(device);
        let bind_group = Self::bind_group(device, &view, &sampler);

        submit_texture(queue, texture.as_image_copy(), pixel);

        Self { texture, bind_group }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

    pub fn bind_group(
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

    // pub fn update_texture(&self, queue: &wgpu::Queue, new_color: Rgb<u8>) {
    //     submit_texture(queue, self.texture.as_image_copy(), new_color.into());
    // }
}

fn submit_texture(
    queue: &wgpu::Queue,
    texture: wgpu::TexelCopyTextureInfo,
    pixel: &Pixel<Rgba<u8>>
) {
    queue.write_texture(
        texture,
        pixel,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * pixel.dimensions().width),
            rows_per_image: Some(pixel.dimensions().height),
        },
        wgpu::Extent3d {
            width: pixel.dimensions().width,
            height: pixel.dimensions().height,
            depth_or_array_layers: 1,
        }
    );
}

fn sampler(device: &wgpu::Device) -> wgpu::Sampler {
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
