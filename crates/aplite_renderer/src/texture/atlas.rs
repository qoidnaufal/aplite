use wgpu::util::DeviceExt;
use aplite_types::{Rect, Size, Vector2};
use super::{ImageData, TextureInfo};

#[derive(Debug)]
pub(crate) struct Atlas {
    used: Rect<u32>,
    texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
    image_data: Vec<(Vector2<u32>, ImageData)>,
    initialized: bool,
    pushed: i32,
}

impl Atlas {
    const SIZE: Size<u32> = Size::new(2000, 2000);

    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let used = Rect::new((0, 0), (0, 0));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture atlas"),
            size: wgpu::Extent3d {
                width: Self::SIZE.width(),
                height: Self::SIZE.height(),
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
        let bind_group = Self::bind_group(device, &view);
        Self {
            used,
            texture,
            bind_group,
            image_data: vec![],
            initialized: false,
            pushed: 0,
        }
    }

    pub(crate) fn push(&mut self, data: ImageData) -> Option<TextureInfo> {
        let width = data.width();
        let height = data.height();

        let is_w_contained = self.used.width() + width <= Self::SIZE.width();
        let is_h_contained = self.used.height() + height <= Self::SIZE.height();

        if is_w_contained && is_h_contained {
            self.used.set_height(
                self.used
                    .height()
                    .max(self.used.y() + height)
            );
        } else if is_h_contained {
            self.used.set_x(0);
            self.used.set_width(0);
            self.used.set_y(self.used.height());
        } else {
            return None;
        }

        let min_x = self.used.l() as f32 / Self::SIZE.width() as f32;
        let min_y = self.used.t() as f32 / Self::SIZE.width() as f32;
        let max_x = width as f32 / Self::SIZE.width() as f32;
        let max_y = height as f32 / Self::SIZE.width() as f32;

        let info = TextureInfo::AtlasId {
            id: self.pushed,
            uv: Rect::new( ( min_x, min_y ), ( max_x, max_y ) ),
        };

        self.image_data.push((self.used.pos(), data));
        self.occupy(width);
        self.pushed += 1;

        Some(info)
    }

    fn occupy(&mut self, width: u32) {
        self.used.add_x(width);
        self.used.add_width(width);
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

    pub(crate) fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if self.image_data.is_empty() { return }

        if !self.initialized {
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let width = Self::SIZE.width() * 4;
            let padding = (align - width % align) % align;
            let padded_width = width + padding;
            let dummy = vec![0_u8; (padded_width * Self::SIZE.height()) as usize];

            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &dummy,
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
                self.texture.as_image_copy(),
                wgpu::Extent3d {
                    width: Self::SIZE.width(),
                    height: Self::SIZE.height(),
                    depth_or_array_layers: 1,
                }
            );
            self.initialized = true;
        }

        for (pos, data) in &self.image_data {
            let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let width = data.width() * 4;
            let padding = (align - width % align) % align;
            let padded_width = width + padding;
            let mut padded_data = Vec::with_capacity((padded_width * data.height()) as usize);

            let mut i = 0;
            for _ in 0..data.height() {
                for _ in 0..width {
                    padded_data.push(data.data[i]);
                    i += 1;
                }
                while (padded_data.len() % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) != 0 {
                    padded_data.push(0);
                }
            }

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
                        x: pos.x(),
                        y: pos.y(),
                        z: 0,
                    },
                },
                wgpu::Extent3d {
                    width: data.width(),
                    height: data.height(),
                    depth_or_array_layers: 1,
                }
            );
        }

        self.image_data.clear();
    }
}

#[cfg(test)]
mod atlas_test {
    use super::*;

    struct Packer {
        max: Size<u32>,
        used: Rect<u32>,
        data: Vec<Rect<u32>>,
    }

    impl Packer {
        fn new() -> Self {
            Self {
                max: Size::<u32>::new(900, 1350),
                used: Rect::<u32>::new((0, 0), (0, 0)),
                data: vec![],
            }
        }

        fn push(&mut self, mut data: Rect<u32>) -> Option<usize> {
            let is_w_contained = self.used.width() + data.width() <= self.max.width();
            let is_h_contained = self.used.height() + data.height() <= self.max.height();

            if is_w_contained && is_h_contained {
                self.used.set_height(self.used.height().max(self.used.y() + data.height()));
            } else if is_h_contained {
                self.used.set_x(0);
                self.used.set_width(0);
                self.used.set_y(self.used.height());
            } else {
                return None;
            }

            data.set_pos(self.used.pos());
            self.used.add_x(data.width());
            self.used.add_width(data.width());

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
            let data = Rect::<u32>::new((0, 0), (450, 450));
            let id = packer.push(data);
            ids.push(id);
        }

        assert_eq!(packer.data.len(), 6);
        assert_eq!(ids[7..].iter().all(|id| id.is_none()), true);
        assert_eq!(
            &packer.data.iter().map(|r| (r.x(), r.y())).collect::<Vec<_>>(),
            &[(0, 0), (450, 0), (0, 450), (450, 450), (0, 900), (450, 900)]
        );
    }
}
