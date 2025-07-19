use std::collections::HashMap;

use wgpu::util::DeviceExt;
use aplite_types::{Rect, Size};

use super::ImageData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtlasId(i32);

impl AtlasId {
    pub(crate) const fn new(id: i32) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub(crate) struct Atlas {
    used: Rect,
    texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,

    pending_data: HashMap<AtlasId, ImageData>,
    position: HashMap<AtlasId, Rect>,
    uvs: HashMap<AtlasId, Rect>,
    count: i32,
}

impl Atlas {
    const SIZE: Size = Size::new(2000., 2000.);

    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let used = Rect::new(0., 0., 0., 0.);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture atlas"),
            size: wgpu::Extent3d {
                width: Self::SIZE.width as u32,
                height: Self::SIZE.height as u32,
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

        // let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        // let width = Self::SIZE.width() * 4;
        // let padding = (align - width % align) % align;
        // let padded_width = width + padding;
        // let data = vec![0_u8; (padded_width * Self::SIZE.height()) as usize];

        // queue.write_texture(
        //     texture.as_image_copy(),
        //     &data,
        //     wgpu::TexelCopyBufferLayout {
        //         offset: 0,
        //         bytes_per_row: Some(padded_width),
        //         rows_per_image: None
        //     },
        //     wgpu::Extent3d {
        //         width: Self::SIZE.width(),
        //         height: Self::SIZE.height(),
        //         depth_or_array_layers: 1,
        //     }
        // );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Self::bind_group(device, &view);

        Self {
            used,
            texture,
            bind_group,
            pending_data: HashMap::new(),
            position: HashMap::new(),
            uvs: HashMap::new(),
            count: 0,
        }
    }

    pub(crate) fn append(&mut self, data: ImageData) -> Option<AtlasId> {
        let width = data.width as f32;
        let height = data.height as f32;

        let is_w_contained = self.used.width + width <= Self::SIZE.width;
        let is_h_contained = self.used.height + height <= Self::SIZE.height;

        if is_w_contained && is_h_contained {
            self.used.height = self.used
                .height
                .max(self.used.y + height);
        } else if is_h_contained {
            self.used.x = 0.;
            self.used.width = 0.;
            self.used.y = self.used.height;
        } else {
            // TODO: double the size?
            return None;
        }

        let min_x = self.used.x / Self::SIZE.width;
        let min_y = self.used.y / Self::SIZE.width;
        let max_x = width / Self::SIZE.width;
        let max_y = height / Self::SIZE.width;

        let resource_id = AtlasId::new(self.count);
        let uv = Rect::new(
            min_x, min_y,
            max_x, max_y
        );

        self.position.insert(resource_id, self.used);
        self.uvs.insert(resource_id, uv);
        self.pending_data.insert(resource_id, data);
        self.occupy(width);
        self.count += 1;

        Some(resource_id)
    }

    fn occupy(&mut self, width: f32) {
        self.used.x += width;
        self.used.width += width;
    }

    pub(crate) fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if !self.pending_data.is_empty() {
            for (id, data) in &self.pending_data {
                let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                let width = data.width * 4;
                let padding = (alignment - width % alignment) % alignment;
                let padded_width = width + padding;
                let mut padded_data = Vec::with_capacity((padded_width * data.height) as usize);

                let mut i = 0;
                for _ in 0..data.height {
                    for _ in 0..width {
                        padded_data.push(data.bytes[i]);
                        i += 1;
                    }
                    while (padded_data.len() % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) != 0 {
                        padded_data.push(0);
                    }
                }

                let pos = self.position[id];
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &padded_data,
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                });

                encoder.copy_buffer_to_texture(
                    wgpu::TexelCopyBufferInfo {
                        buffer: &buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(padded_width),
                            rows_per_image: None,
                        },
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.texture,
                        aspect: wgpu::TextureAspect::All,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: pos.x as u32,
                            y: pos.y as u32,
                            z: 0,
                        },
                    },
                    wgpu::Extent3d {
                        width: data.width,
                        height: data.height,
                        depth_or_array_layers: 1,
                    }
                );
            }

            self.pending_data.clear();
        }
    }

    pub(crate) fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atlas bind group layout"),
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
            label: Some("atlas bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        })
    }

    #[inline(always)]
    pub(crate) fn get_uv(&self, id: &AtlasId) -> Option<&Rect> {
        self.uvs.get(id)
    }
}

#[cfg(test)]
mod atlas_test {
    use super::*;

    struct Packer {
        max: Size,
        used: Rect,
        data: Vec<Rect>,
    }

    impl Packer {
        fn new() -> Self {
            Self {
                max: Size::new(900., 1350.),
                used: Rect::new(0., 0., 0., 0.),
                data: vec![],
            }
        }

        fn push(&mut self, mut data: Rect) -> Option<usize> {
            let is_w_contained = self.used.width + data.width <= self.max.width;
            let is_h_contained = self.used.height + data.height <= self.max.height;

            if is_w_contained && is_h_contained {
                self.used.height = self.used.height.max(self.used.y + data.height);
            } else if is_h_contained {
                self.used.x = 0.;
                self.used.width = 0.;
                self.used.y = self.used.height;
            } else {
                return None;
            }

            data.set_pos(self.used.pos());
            self.used.x += data.width;
            self.used.width += data.width;

            let id = self.data.len();
            self.data.push(data);
            Some(id)
        }
    }

    #[test]
    fn packing() {
        let mut packer = Packer::new();
        let mut ids = vec![];
        for _ in 0..8 {
            let data = Rect::new(0., 0., 450., 450.);
            let id = packer.push(data);
            ids.push(id);
        }

        assert_eq!(packer.data.len(), 6);
        assert_eq!(ids[7..].iter().all(|id| id.is_none()), true);
        assert_eq!(
            &packer.data.iter().map(|r| (r.x, r.y)).collect::<Vec<_>>(),
            &[(0., 0.), (450., 0.), (0., 450.), (450., 450.), (0., 900.), (450., 900.)]
        );
    }
}
