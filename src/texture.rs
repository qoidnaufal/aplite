use math::Size;

use crate::pipeline::bind_group;

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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
        let data = TextureData::new(device, bg_layout, uniform, size);
        Self { data, used_size: Size::new(0, 0) }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.data.bind_group
    }

    pub fn push(
        &mut self,
        queue: &wgpu::Queue,
        uv_size: Size<u32>,
        uv_data: &[u8],
    ) {
        assert!(self.used_size < self.data.size);
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.data.texture,
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

        self.used_size += uv_size;
    }
}
