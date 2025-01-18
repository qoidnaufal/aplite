use std::path::Path;
use std::io::{BufReader, Read};
use std::fs::File;

use image::GenericImageView;
use math::Size;

use crate::pipeline::bind_group;

pub fn image_reader<P: AsRef<Path>>(path: P) -> ImageData {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    let len = reader.read_to_end(&mut buf).unwrap();

    let image = image::load_from_memory(&buf[..len]).unwrap();

    ImageData {
        dimension: image.dimensions().into(),
        data: image.to_rgba8().to_vec(),
    }
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub dimension: Size<u32>,
    pub data: Vec<u8>,
}

struct TextureData {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    size: Size<u32>,
}

impl TextureData {
    fn new(
        device: &wgpu::Device,
        bg_layout: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        size: Size<u32>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = sampler(device);
        let bind_group = bind_group(device, bg_layout, &view, &sampler, uniform);

        Self { texture, bind_group, size }
    }
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

pub struct TextureCollection {
    data: TextureData,
    used_size: Size<u32>,
}

impl TextureCollection {
    pub fn new(
        device: &wgpu::Device,
        bg_layout: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        size: Size<u32>,
    ) -> Self {
        Self {
            data: TextureData::new(device, bg_layout, uniform, size),
            used_size: Size::new(0, 0)
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.data.bind_group
    }

    pub fn push(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
        uniform: &wgpu::Buffer,
        uv_size: Size<u32>,
        uv_data: &[u8],
    ) {
        let new_size = self.data.size + uv_size;
        let dst = TextureData::new(device, bg_layout, uniform, new_size);
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_texture(
            self.data.texture.as_image_copy(),
            dst.texture.as_image_copy(),
            self.data.texture.size()
        );
        queue.submit(std::iter::once(encoder.finish()));
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &dst.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: self.used_size.height,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            uv_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * uv_size.width),
                rows_per_image: Some(uv_size.height),
            },
            wgpu::Extent3d {
                width: uv_size.width,
                height: uv_size.height,
                depth_or_array_layers: 1,
            }
        );

        self.data = dst;
        self.used_size += uv_size;
    }
}
