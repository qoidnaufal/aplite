use std::path::Path;
use std::io::{BufReader, Read};
use std::fs::File;

use image::GenericImageView;

use crate::renderer::Buffer;
use crate::renderer::bind_group;
use crate::color::{Color, Rgb};
use crate::shapes::Transform;
use crate::Rgba;

pub fn image_reader<P: AsRef<Path>>(path: P) -> Color<Rgba<u8>, u8> {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    let len = reader.read_to_end(&mut buf).unwrap();

    let image = image::load_from_memory(&buf[..len]).unwrap();

    Color::new(image.dimensions(), &image.to_rgba8())
}

#[derive(Debug)]
pub struct TextureData {
    texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    pub u_buffer: Buffer<Transform>,
}

impl TextureData {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
        u_buffer: Buffer<Transform>,
        data: Color<Rgba<u8>, u8>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: data.dimensions().width,
                height: data.dimensions().height,
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
        let bind_group = bind_group(device, bg_layout, &view, &sampler, &u_buffer.buffer);

        submit_texture(queue, texture.as_image_copy(), data);

        Self { texture, bind_group, u_buffer }
    }

    pub fn change_color(&self, queue: &wgpu::Queue, new_color: Rgb<u8>) {
        // FIXME: texture size checking
        submit_texture(queue, self.texture.as_image_copy(), new_color.into());
    }
}

fn submit_texture(queue: &wgpu::Queue, texture: wgpu::TexelCopyTextureInfo, data: Color<Rgba<u8>, u8>) {
    queue.write_texture(
        texture,
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * data.dimensions().width),
            rows_per_image: Some(data.dimensions().height),
        },
        wgpu::Extent3d {
            width: data.dimensions().width,
            height: data.dimensions().height,
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
