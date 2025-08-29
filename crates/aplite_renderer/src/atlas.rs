use std::collections::HashMap;

use wgpu::util::DeviceExt;

use aplite_types::{Rect, Size, Vec2f, ImageRef};
use aplite_storage::{IndexMap, Tree, Entity, entity};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uv {
    pub(crate) min_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_x: f32,
    pub(crate) max_y: f32,
}

pub(crate) struct Atlas {
    texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,

    allocator: AtlasAllocator,
    pending_data: Vec<(Rect, ImageRef)>,
    processed: HashMap<ImageRef, Uv>,
}

impl Atlas {
    pub(crate) fn new(device: &wgpu::Device, size: Size) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture atlas"),
            size: wgpu::Extent3d {
                width: size.width as u32,
                height: size.height as u32,
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
            allocator: AtlasAllocator::new(size),
            texture,
            bind_group,
            pending_data: Vec::new(),
            processed: HashMap::new(),
        }
    }

    pub(crate) fn append(&mut self, data: &ImageRef) -> Option<Uv> {
        if let Some(uv) = self.processed.get(data) {
            return Some(*uv)
        }

        let size = Size::new(data.width as _, data.height as _);
        let allocated = self.allocator.alloc(size);

        // if let Some(rect) = allocated {
        //     self.pending_data.push((rect, data.clone()));
        // }

        allocated.map(|rect| {
            let min_x = rect.x / self.allocator.rect.width;
            let min_y = rect.y / self.allocator.rect.height;
            let max_x = min_x + rect.width / self.allocator.rect.width;
            let max_y = min_y + rect.height / self.allocator.rect.height;

            self.pending_data.push((rect, data.clone()));

            Uv {
                min_x,
                min_y,
                max_x,
                max_y,
            }
        })
    }

    // #[inline(always)]
    // pub(crate) fn get_uv(&self, id: &AtlasId) -> Option<Rect> {
    //     self.allocator
    //         .get_rect(id)
    //         .map(|rect| {
    //             let min_x = rect.x / self.allocator.rect.width;
    //             let min_y = rect.y / self.allocator.rect.width;
    //             let max_x = rect.width / self.allocator.rect.width;
    //             let max_y = rect.height / self.allocator.rect.width;

    //             Rect::new(
    //                 min_x, min_y,
    //                 max_x, max_y
    //             )
    //         })
    // }

    pub(crate) fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if !self.pending_data.is_empty() {
            self.pending_data
                .drain(..)
                .for_each(|(rect, data)| {
                    let data = data.upgrade().expect("Image Data shouldn't have been removed");
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
                                x: rect.x as u32,
                                y: rect.y as u32,
                                z: 0,
                            },
                        },
                        wgpu::Extent3d {
                            width: data.width,
                            height: data.height,
                            depth_or_array_layers: 1,
                        }
                    );

                    let min_x = rect.x / self.allocator.rect.width;
                    let min_y = rect.y / self.allocator.rect.height;
                    let max_x = min_x + (rect.width / self.allocator.rect.width);
                    let max_y = min_y + (rect.height / self.allocator.rect.height);

                    let uv = Uv {
                        min_x,
                        min_y,
                        max_x,
                        max_y,
                    };

                    self.processed.insert(data.downgrade(), uv);
                });
        } else if self.pending_data.capacity() > 0 {
            self.pending_data.shrink_to(0);
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
}

entity! { pub AtlasId }

struct AtlasAllocator {
    rect: Rect,
    last_parent: Option<AtlasId>,
    allocated: IndexMap<AtlasId, Rect>,
    tree: Tree<AtlasId>,
}

impl AtlasAllocator {
    fn new(size: Size) -> Self {
        Self {
            rect: Rect::from_size(size),
            last_parent: None,
            allocated: IndexMap::new(),
            tree: Tree::default(),
        }
    }

    fn alloc(&mut self, size: Size) -> Option<Rect> {
        // TODO: double the size
        if size.area() > self.calculate_available_area() { return None };

        match self.last_parent {
            Some(last_parent) => {
                if let Some((parent, pos)) = self.scan(size) {
                    let rect = Rect::from_vec2f_size(pos, size);
                    let id = self.allocated.insert(rect);

                    self.tree.add_child(parent, id);

                    Some(rect)
                } else {
                    // inserting as the next sibling of the last parent
                    let next_y = self.allocated.get(last_parent).unwrap().max_y();
                    let pos = Vec2f::new(0.0, next_y);
                    let rect = Rect::from_vec2f_size(pos, size);
                    let id = self.allocated.insert(rect);

                    self.tree.add_sibling(last_parent, id);
                    // self.tree.add_root(id);
                    self.last_parent = Some(id);

                    Some(rect)
                }
            },
            None => {
                // first insert
                let rect = Rect::from_size(size);
                let id = self.allocated.insert(rect);

                self.tree.add_root(id);
                self.last_parent = Some(id);

                Some(rect)
            },
        }
    }

    fn scan(&self, size: Size) -> Option<(AtlasId, Vec2f)> {
        self.get_parents()
            .find_map(|(id, rect)| self.identify_children(id, rect, size))
    }

    fn identify_children(
        &self,
        parent: AtlasId,
        rect: &Rect,
        size: Size,
    ) -> Option<(AtlasId, Vec2f)> {
        if let Some(first) = self.tree.get_first_child(parent) {
            let mut current = first;

            while let Some(sibling) = self.tree.get_next_sibling(current) {
                let find = match self.tree.get_last_child(current) {
                    Some(last) => self.check_pos_for(last, current, size),
                    None => self.check_pos_for(current, current, size),
                }
                .or(self.indentify_next_sibling(current, parent, size));

                if find.is_some() {
                    return find
                } else {
                    current = sibling;
                }
            }

            None
        } else {
            (rect.max_x() + size.width <= self.rect.width)
                .then_some((
                    parent,
                    Vec2f::new(rect.max_x(), rect.y)
                ))
        }

        // initial implementation which created unnecessary allocation on Tree::get_all_children
        // match self.tree.get_all_children(&parent) {
        //     Some(children) => {
        //         children
        //             .iter()
        //             .find_map(|child_id| {
        //                 match self.tree.get_last_child(child_id) {
        //                     Some(last) => self.check_pos_for(last, child_id, size),
        //                     None => self.check_pos_for(child_id, child_id, size),
        //                 }
        //                 .or(self.indentify_next_sibling(child_id, &parent, size))
        //             })
        //     },
        //     None => (rect.max_x() + size.width <= self.rect.width)
        //         .then_some((
        //             parent,
        //             Vec2f::new(rect.max_x(), rect.y)
        //         )),
        // }
    }

    #[inline(always)]
    fn check_pos_for(
        &self,
        id: AtlasId,
        parent: AtlasId,
        size: Size,
    ) -> Option<(AtlasId, Vec2f)> {
        let rect = self.allocated.get(id).unwrap();
        let cond1 = rect.max_y() + size.height <= rect.max_y();
        let cond2 = size.width <= rect.width;

        (cond1 && cond2).then_some((
            parent,
            Vec2f::new(rect.x, rect.max_y())
        ))
    }

    #[inline(always)]
    fn indentify_next_sibling(
        &self,
        prev: AtlasId,
        parent: AtlasId,
        size: Size,
    ) -> Option<(AtlasId, Vec2f)> {
        let rect = self.allocated.get(prev).unwrap();
        let cond1 = rect.max_x() + size.width <= self.rect.width;
        let cond2 = self.tree.get_next_sibling(prev).is_none();

        (cond1 && cond2).then_some((
            parent,
            Vec2f::new(rect.max_x(), rect.y)
        ))
    }

    #[inline(always)]
    fn get_parents(&self) -> impl Iterator<Item = (AtlasId, &Rect)> {
        self.allocated
            .iter()
            .filter_map(|(id, rect)| {
                self.tree
                    .get_parent(id)
                    .is_none()
                    .then_some((id, rect))
            })
    }

    // fn remove(&mut self, id: AtlasId) -> Option<Rect> {
    //     // shifting
    //     if let Some(prev) = self.get_prev_sibling(&id).copied() {
    //         self.next_sibling[prev.index()] = self.get_next_sibling(&id).copied();
    //     } else if let Some(parent) = self.get_parent(&id).copied() {
    //         self.first_child[parent.index()] = self.get_next_sibling(&id).copied();
    //     }
        
    //     self.allocated.remove(&id)
    // }

    #[inline(always)]
    fn calculate_available_area(&self) -> f32 {
        let allocated = self.allocated
            .iter()
            .fold(0.0,|sum, (_, rect)| {
                sum + rect.area()
            });
        self.rect.area() - allocated
    }
}

#[cfg(test)]
mod atlas_test {
    use super::*;

    #[test]
    fn packing() {
        let mut allocator = AtlasAllocator::new(Size::new(700., 1000.));

        let first = allocator.alloc(Size::new(500., 200.));
        assert!(first.is_some());

        let second = allocator.alloc(Size::new(500., 200.));
        assert!(second.is_some());
        assert_eq!(second.unwrap().vec2f(), Vec2f::new(0., 200.));

        let third = allocator.alloc(Size::new(100., 100.));
        assert!(third.is_some());
        // assert_eq!(allocator.get_first_child(&first.unwrap()), third.as_ref());

        let fourth = allocator.alloc(Size::new(100., 100.));
        assert!(fourth.is_some());
        // assert_eq!(allocator.get_last_child(&third.unwrap()), fourth.as_ref());

        let fifth = allocator.alloc(Size::new(100., 100.));
        assert!(fifth.is_some());
        // assert_eq!(allocator.get_last_child(&first.unwrap()), fifth.as_ref());
        // assert_eq!(allocator.get_next_sibling(&third.unwrap()), fifth.as_ref());

        let sixth = allocator.alloc(Size::new(300., 100.));
        assert!(sixth.is_some());
        // assert_eq!(allocator.get_next_sibling(&second.unwrap()), sixth.as_ref());
        assert_eq!(allocator.get_parents().count(), 3);

        eprintln!("{:#?}", allocator.tree);
    }
}
