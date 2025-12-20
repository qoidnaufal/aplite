use std::collections::HashMap;

// use wgpu::util::DeviceExt;

use aplite_types::{Rect, Size, Point, ImageRef};
use aplite_storage::{SparseTree, EntityManager, EntityId, EntityIdMap};

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
            let min_x = rect.x / self.allocator.bound.width;
            let min_y = rect.y / self.allocator.bound.height;
            let max_x = min_x + rect.width / self.allocator.bound.width;
            let max_y = min_y + rect.height / self.allocator.bound.height;

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
    // pub(crate) fn get_uv(&self, id: &EntityId) -> Option<Rect> {
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

    pub(crate) fn update(&mut self, queue: &wgpu::Queue) {
        if !self.pending_data.is_empty() {
            std::mem::take(&mut self.pending_data)
                .iter()
                .for_each(|(rect, data)| {
                    let data = data.upgrade().expect("ImageData need to be alive for rendering");
                    // let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                    // let width = data.width * 4;
                    // let padding = (alignment - width % alignment) % alignment;
                    // let padded_width = width + padding;
                    // let mut padded_data = Vec::with_capacity((padded_width * data.height) as usize);

                    queue.write_texture(
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
                        &data.bytes,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: None,
                            rows_per_image: None,
                        },
                        wgpu::Extent3d {
                            width: data.width,
                            height: data.height,
                            depth_or_array_layers: 1,
                        }
                    );

                    // let mut i = 0;
                    // for _ in 0..data.height {
                    //     for _ in 0..width {
                    //         padded_data.push(data.bytes[i]);
                    //         i += 1;
                    //     }
                    //     while (padded_data.len() % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) != 0 {
                    //         padded_data.push(0);
                    //     }
                    // }

                    // let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    //     label: None,
                    //     contents: &padded_data,
                    //     usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
                    // });

                    // encoder.copy_buffer_to_texture(
                    //     wgpu::TexelCopyBufferInfo {
                    //         buffer: &buffer,
                    //         layout: wgpu::TexelCopyBufferLayout {
                    //             offset: 0,
                    //             bytes_per_row: Some(padded_width),
                    //             rows_per_image: None,
                    //         },
                    //     },
                    //     wgpu::TexelCopyTextureInfo {
                    //         texture: &self.texture,
                    //         aspect: wgpu::TextureAspect::All,
                    //         mip_level: 0,
                    //         origin: wgpu::Origin3d {
                    //             x: rect.x as u32,
                    //             y: rect.y as u32,
                    //             z: 0,
                    //         },
                    //     },
                    //     wgpu::Extent3d {
                    //         width: data.width,
                    //         height: data.height,
                    //         depth_or_array_layers: 1,
                    //     }
                    // );

                    let min_x = rect.x / self.allocator.bound.width;
                    let min_y = rect.y / self.allocator.bound.height;
                    let max_x = min_x + (rect.width / self.allocator.bound.width);
                    let max_y = min_y + (rect.height / self.allocator.bound.height);

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

// ┬ ┴ ├ ┤ ┼ ┌ ┐ └ ┘ │ ─
// ↓ →

/// The priority is to fill the atlas horizontally first from each root.
/// The root will be placed on the left-most and stacked vertically.
/// Once a root is assigned, horizontally is children, vertically is siblings.
/// # graphical representation
/// ┌───────────┬───────────────────────┐
/// │           →      first child      → child of first child
/// │   Root0   ├───────────↓───────────┤
/// │           │siblings of first child│
/// ├─────↓─────┼───────────────────────┤
/// │           │                       │
/// │   Root1   →                       │
/// │           │                       │
/// └───────────┴───────────────────────┘
/// The first child will set the boundary for it's siblings if any, and the children of it's siblings.
/// This means the total width of the first child's siblings + their childrens <= first child's rect
struct AtlasAllocator {
    bound: Rect,
    last_root: Option<EntityId>,
    id_manager: EntityManager,
    allocated: EntityIdMap<Rect>,
    tree: SparseTree,
}

impl AtlasAllocator {
    fn new(size: impl Into<Size>) -> Self {
        Self {
            bound: Rect::from_size(size.into()),
            last_root: None,
            id_manager: EntityManager::default(),
            allocated: HashMap::default(),
            tree: SparseTree::default(),
        }
    }

    // FIXME: there's some logic error here
    fn alloc(&mut self, new_size: Size) -> Option<Rect> {
        // TODO: double the size
        if new_size.area() > self.calculate_available_area() { return None };

        match self.last_root {
            Some(last_root) => {
                if let Some((parent, pos)) = self.scan(new_size) {
                    let rect = Rect::from_point_size(pos, new_size);
                    let id = self.id_manager.create().id();

                    self.allocated.insert(id, rect);
                    self.tree.insert_with_parent(id, parent);

                    Some(rect)
                } else {
                    // inserting as the next root
                    let next_y = self.allocated.get(&last_root).unwrap().max_y();
                    let pos = Point::new(0.0, next_y);
                    let rect = Rect::from_point_size(pos, new_size);
                    let id = self.id_manager.create().id();

                    self.allocated.insert(id, rect);
                    self.tree.insert_as_root(id);
                    self.last_root = Some(id);

                    Some(rect)
                }
            },
            None => {
                // first insert
                let rect = Rect::from_size(new_size);
                let id = self.id_manager.create().id();

                self.allocated.insert(id, rect);
                self.tree.insert_as_root(id);
                self.last_root = Some(id);
        
                Some(rect)
            },
        }
    }

    #[inline(always)]
    /// scan each roots and try to find available position within the identified root
    fn scan(&self, new_size: Size) -> Option<(EntityId, Point)> {
        self.iter_roots()
            .find_map(|(root, bound_rect)| self.identify_member(root, bound_rect, new_size))
    }

    #[inline(always)]
    fn iter_roots<'a>(&'a self) -> impl Iterator<Item = (EntityId, &'a Rect)> {
        self.tree
            .roots()
            .filter_map(|id| {
                self.allocated
                    .get(&id)
                    .map(|rect| (id, rect))
            })
    }

    #[inline(always)]
    fn identify_member(
        &self,
        root: EntityId,
        bound_rect: &Rect,
        new_size: Size,
    ) -> Option<(EntityId, Point)> {
        if let Some(first) = self.tree.get_first_child(root) {
            // the first rect will set the boundary of it's siblings if any
            let first_rect = self.allocated.get(&first).unwrap();

            let mut current = first;

            while let Some(sibling) = self.tree.get_next_sibling(current) {
                let find = self.indentify_next_sibling(sibling, first_rect, new_size);
                if find.is_some() { return find }
                current = sibling;
            }

            // assigning as the next sibling of the first child
            let last_rect = self.allocated.get(&current).unwrap();

            (new_size.width <= first_rect.width
                && new_size.height + last_rect.max_y() <= bound_rect.height)
                    .then_some((
                        root,
                        Point::new(last_rect.x, last_rect.max_y())
                    ))
                    .or_else(|| {
                        (new_size.width + first_rect.max_x() <= self.bound.width
                            && new_size.height <= bound_rect.height)
                                .then_some((
                                    first,
                                    Point::new(first_rect.max_x(), bound_rect.y)
                                ))
                    })
        } else {
            // assign as the first child of a root if fit
            (bound_rect.max_x() + new_size.width <= self.bound.width)
                .then_some((
                    root,
                    Point::new(bound_rect.max_x(), bound_rect.y)
                ))
        }
    }

    /// Identify the possibility of becoming a child of the siblings of root's first-child
    #[inline(always)]
    fn indentify_next_sibling(
        &self,
        current: EntityId,
        first_rect_bound: &Rect,
        new_size: Size,
    ) -> Option<(EntityId, Point)> {
        let current_rect = self.allocated.get(&current).unwrap();

        let cond1 = new_size.width + current_rect.max_x() <= first_rect_bound.max_x();
        let cond2 = new_size.height <= current_rect.height;

        if let Some(cfc) = self.tree.get_first_child(current) {
            let cfc_rect = self.allocated.get(&cfc).unwrap();

            let mut curr = cfc;
            while let Some(next) = self.tree.get_next_sibling(curr) {
                let find = self.indentify_next_sibling(next, cfc_rect, new_size);
                if find.is_some() { return find }
                curr = next;
            }
        }

        (cond1 && cond2).then_some((
            current,
            Point::new(current_rect.max_x(), current_rect.y)
        ))
    }

    // fn remove(&mut self, id: EntityId) -> Option<Rect> {
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

        self.bound.area() - allocated
    }
}

#[cfg(test)]
mod atlas_test {
    use super::*;

    #[test]
    fn atlas_allocator() {
        let mut allocator = AtlasAllocator::new((700, 1000));

        // parent 0
        let zero = allocator.alloc(Size::new(400., 300.));
        assert!(zero.is_some());

        // parent 1
        let one = allocator.alloc(Size::new(400., 300.));
        assert!(one.is_some());
        assert_eq!(one.unwrap().point(), Point::new(0., 300.));

        let two = allocator.alloc(Size::new(300., 100.));
        assert!(two.is_some());

        let three = allocator.alloc(Size::new(300., 100.));
        assert!(three.is_some());

        let four = allocator.alloc(Size::new(100., 100.));
        assert!(four.is_some());

        // // parent 2
        let five = allocator.alloc(Size::new(500., 100.));
        assert!(five.is_some());

        let six = allocator.alloc(Size::new(200., 50.));
        assert!(six.is_some());

        let seven = allocator.alloc(Size::new(50., 50.));
        assert!(seven.is_some());

        let eight = allocator.alloc(Size::new(50., 50.));
        assert!(eight.is_some());

        assert_eq!(allocator.tree.roots().count(), 3);

        eprintln!("{:#?}", allocator.tree);

        // > EntityId(0)
        //   ├─ EntityId(2)
        //   ├─ EntityId(3)
        //   └─ EntityId(4)
        //      ├─ EntityId(6)
        //      └─ EntityId(7)
        //         └─ EntityId(8)
        // > EntityId(1)
        // > EntityId(5)
    }
}
