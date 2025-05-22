use std::path::Path;

use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use shared::{Fraction, Rect, Rgba, Size};

use super::Gpu;

pub fn image_reader<P: AsRef<Path>>(path: P) -> ImageData {
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .resize_to_fill(450, 450, FilterType::Lanczos3);

    ImageData::new(img.dimensions(), &img.to_rgba8())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    rect: Rect<u32>,
    data: Vec<u8>,
}

impl ImageData {
    pub fn new(size: impl Into<Size<u32>>, data: &[u8]) -> Self {
        let size: Size<u32> = size.into();
        Self {
            rect: Rect::new((0, 0), (size.width(), size.height())),
            data: data.to_vec(),
        }
    }

    pub const fn rect(&self) -> Rect<u32> { self.rect }

    pub fn aspect_ratio(&self) -> Fraction<u32> {
        self.rect.size().aspect_ratio()
    }

    pub(crate) const fn size(&self) -> Size<u32> { self.rect.size() }

    pub(crate) const fn width(&self) -> u32 { self.rect.width() }

    pub(crate) const fn height(&self) -> u32 { self.rect.height() }

    // pub(crate) const fn x(&self) -> u32 { self.rect.x() }

    // pub(crate) const fn y(&self) -> u32 { self.rect.y() }
}

impl std::ops::Deref for ImageData {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl From<Rgba<u8>> for ImageData {
    fn from(rgba: Rgba<u8>) -> Self {
        Self::new((1, 1), &rgba.to_slice())
    }
}

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

pub(crate) mod atlas {
    use wgpu::util::DeviceExt;
    use shared::{Rect, Size, Vector2, Fraction};
    use super::ImageData;

    #[derive(Debug)]
    pub(crate) struct TextureInfo {
        pub(crate) id: i32,
        pub(crate) aspect_ratio: Fraction<u32>,
        pub(crate) rect: Rect<f32>,
    }

    #[derive(Debug)]
    pub(crate) struct Atlas {
        used: Rect<u32>,
        texture: wgpu::Texture,
        pub(crate) bind_group: wgpu::BindGroup,
        image_data: Vec<ImageData>,
        initialized: bool,
        pushed: i32,
    }

    impl Atlas {
        const SIZE: Size<u32> = Size::new(1024, 1024);
        const SPACING: u32 = 1;

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
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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

        pub(crate) fn push_image(&mut self, mut data: ImageData) -> Option<TextureInfo> {
            let is_w_contained = self.used.width() + data.width() <= Self::SIZE.width();
            let is_h_contained = self.used.height() + data.height() <= Self::SIZE.height();

            if is_w_contained && is_h_contained {
                self.used
                    .set_height(
                        self.used
                            .height()
                            .max(self.used.y() + data.height())
                    );
            } else if is_h_contained {
                self.used.set_x(0);
                self.used.set_width(0);
                self.used.set_y(self.used.height() + Self::SPACING);
            } else {
                return None;
            }

            data.rect.set_pos(self.used.pos());
            self.used.add_x(data.width() + Self::SPACING);
            self.used.add_width(data.width() + Self::SPACING);

            let mut rect: Rect<f32> = data.rect.into();
            rect.set_pos(rect.pos() / Self::SIZE.width() as f32);
            rect.set_size(rect.size() / Self::SIZE.width() as f32);
            let info = TextureInfo {
                id: self.pushed,
                aspect_ratio: data.aspect_ratio(),
                rect,
            };
            self.image_data.push(data);
            self.pushed += 1;
            Some(info)
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

            for data in &self.image_data {
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
                            x: data.rect.x(),
                            y: data.rect.y(),
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
}
