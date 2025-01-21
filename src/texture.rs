use std::path::Path;
use std::io::{BufReader, Read};
use std::fs::File;

use image::GenericImageView;
use math::Size;

use crate::renderer::Buffer;
use crate::renderer::bind_group;
use crate::color::{Color, Rgb};
use crate::shapes::Transform;
use crate::NodeId;

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

#[derive(Debug)]
pub struct TextureData {
    pub node_id: NodeId,
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
        size: Size<u32>,
        uv_data: &[u8],
        node_id: NodeId,
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
        let bind_group = bind_group(device, bg_layout, &view, &sampler, &u_buffer.buffer);

        submit_texture(queue, texture.as_image_copy(), size, uv_data);

        Self { node_id, texture, bind_group, u_buffer }
    }

    pub fn change_color(&self, queue: &wgpu::Queue, new_color: Rgb<u8>) {
        let size = Size::new(1, 1);
        let color_data = Color::from(new_color);
        submit_texture(queue, self.texture.as_image_copy(), size, &color_data);
    }
}

fn submit_texture(queue: &wgpu::Queue, texture: wgpu::ImageCopyTexture, uv_size: Size<u32>, uv_data: &[u8]) {
    queue.write_texture(
        texture,
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
