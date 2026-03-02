use std::sync::{Arc, Weak};

use rustc_hash::FxHashMap;

use aplite_types::{Rect, Size, Point};
use aplite_storage::{SparseTree, SlotMap, SlotId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Uv {
    pub(crate) min_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_x: f32,
    pub(crate) max_y: f32,
}

impl Uv {
    pub(crate) const DEFAULT: Self = Self {
        min_x: 0.,
        min_y: 0.,
        max_x: 1.,
        max_y: 1.,
    };
}

#[derive(Clone)]
pub struct TextureRef {
    pub width: u32,
    pub height: u32,
    pub bytes: Weak<[u8]>,
}

impl TextureRef {
    pub fn new(width: u32, height: u32, bytes: Weak<[u8]>) -> Self {
        Self {
            width,
            height,
            bytes,
        }
    }

    pub fn upgrade(&self) -> Option<TextureData> {
        self.bytes.upgrade()
            .map(|bytes| TextureData {
                width: self.width,
                height: self.height,
                bytes,
            })
    }
}

impl PartialEq for TextureRef {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.bytes, &other.bytes)
    }
}

impl Eq for TextureRef {}

impl std::hash::Hash for TextureRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // if let Some(data) = self.upgrade() {
        //     state.write(data.bytes.as_ref());
        // }
        state.write_usize(Weak::as_ptr(&self.bytes).addr());
    }
}

#[derive(Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub bytes: Arc<[u8]>,
}

impl TextureData {
    pub fn new(width: u32, height: u32, bytes: Arc<[u8]>) -> Self {
        Self {
            width,
            height,
            bytes,
        }
    }

    pub fn downgrade(&self) -> TextureRef {
        TextureRef {
            width: self.width,
            height: self.height,
            bytes: Arc::downgrade(&self.bytes),
        }
    }
}

pub(crate) struct Atlas {
    texture: wgpu::Texture,

    pub(crate) allocator: AtlasAllocator,
    pending_data: Vec<(Rect, TextureRef)>,
    processed: FxHashMap<TextureRef, Uv>,
}

impl Atlas {
    pub(crate) fn new(device: &wgpu::Device, size: Size, label: &str) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
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

        Self {
            allocator: AtlasAllocator::new(size),
            texture,
            pending_data: Vec::new(),
            processed: FxHashMap::default(),
        }
    }

    pub(crate) fn view(&self) -> wgpu::TextureView {
        self.texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub(crate) fn append(&mut self, data: &TextureRef) -> Option<Uv> {
        if let Some(uv) = self.processed.get(data) {
            return Some(*uv)
        }

        let size = Size::new(data.width as _, data.height as _);

        self.allocator.alloc(size).map(|rect| {
            self.pending_data.push((rect, data.clone()));
            self.allocator.get_uv(rect)
        })
    }

    pub(crate) fn update(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        use wgpu::util::DeviceExt;
        use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as ALIGN;

        if !self.pending_data.is_empty() {
            std::mem::take(&mut self.pending_data)
                .into_iter()
                .for_each(|(rect, pending_data)| {
                    if let Some(data) = pending_data.upgrade() {
                        let width = data.width * 4;
                        let padding = (ALIGN - width % ALIGN) % ALIGN;
                        let padded_width = width + padding;

                        let mut padded_data = Vec::with_capacity((padded_width * data.height) as usize);

                        let mut i = 0;

                        for _ in 0..data.height {
                            for _ in 0..data.width {
                                padded_data.push(data.bytes[i]);
                                i += 1;
                            }
                            while (padded_data.len() % ALIGN as usize) != 0 {
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

                        let uv = self.allocator.get_uv(rect);

                        self.processed.insert(pending_data, uv);
                    }
                });
        }
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
pub(crate) struct AtlasAllocator {
    pub(crate) bound: Rect,
    last_root: Option<SlotId>,
    allocated: SlotMap<Rect>,
    pub(crate) tree: SparseTree,
}

impl AtlasAllocator {
    pub(crate) fn new(size: impl Into<Size>) -> Self {
        Self {
            bound: Rect::from_size(size.into()),
            last_root: None,
            allocated: SlotMap::new(),
            tree: SparseTree::default(),
        }
    }

    pub(crate) fn alloc(&mut self, new_size: Size) -> Option<Rect> {
        // TODO: double the size
        if new_size.area() > self.calculate_available_area() { return None };

        match self.last_root {
            Some(last_root) => {
                if let Some((parent, pos)) = self.scan(new_size) {
                    let rect = Rect::from_point_size(pos, new_size);

                    self.allocated.try_insert(rect).ok().map(|id| {
                        self.tree.insert_with_parent(id, parent);
                        rect
                    })

                } else {
                    // inserting as the next root
                    let next_y = unsafe {
                        self.allocated.get_unchecked(&last_root).max_y()
                    };

                    let pos = Point::new(0.0, next_y);
                    let rect = Rect::from_point_size(pos, new_size);

                    self.allocated.try_insert(rect).ok().map(|id| {
                        self.tree.insert_as_root(id);
                        self.last_root = Some(id);
                        rect
                    })
                }
            },
            None => {
                // first insert
                let rect = Rect::from_size(new_size);

                self.allocated.try_insert(rect).ok().map(|id| {
                    self.tree.insert_as_root(id);
                    self.last_root = Some(id);
                    rect
                })
            },
        }
    }

    /// scan each roots and try to find available position within the identified root
    fn scan(&self, new_size: Size) -> Option<(SlotId, Point)> {
        self.tree.roots().find_map(|root| unsafe {
            let rect = self.allocated.get_unchecked(&root);
            self.identify_member(root, rect, new_size)
        })
    }

    fn identify_member(
        &self,
        root: SlotId,
        root_rect: &Rect,
        new_size: Size,
    ) -> Option<(SlotId, Point)> {
        if let Some(first) = self.tree.get_first_child(root) {
            // the first rect will set the boundary of it's siblings if any
            let first_rect = unsafe { self.allocated.get_unchecked(&first) };

            let mut current = first;

            while let Some(sibling) = self.tree.get_next_sibling(current) {
                let find = self.indentify_next_sibling(sibling, first_rect, new_size);
                if find.is_some() { return find }
                current = sibling;
            }

            // assigning as the next sibling / child of the first child
            let last_rect = unsafe { self.allocated.get_unchecked(&current) };

            if new_size.width <= first_rect.width
                && new_size.height + last_rect.max_y() <= root_rect.height
            {
                Some((root, Point::new(last_rect.x, last_rect.max_y())))
            } else if new_size.width + first_rect.max_x() <= self.bound.width
                && new_size.height <= root_rect.height
            {
                if let Some(lc) = self.tree.get_last_child(first) {
                    let lr = unsafe { self.allocated.get_unchecked(&lc) };
                    let mut curr = lc;
                    while let Some(sib) = self.tree.get_next_sibling(curr) {
                        let find = self.indentify_next_sibling(sib, lr, new_size);
                        if find.is_some() { return find }
                        curr = sib;
                    }
                    self.identify_member(curr, lr, new_size)
                } else {
                    Some((first, Point::new(first_rect.max_x(), root_rect.y)))
                }
            } else {
                None
            }
        } else {
            // assign as the first child of a root if fit
            if new_size.width + root_rect.max_x() <= self.bound.width
                && root_rect.height <= new_size.height
            {
                Some((root, Point::new(root_rect.max_x(), root_rect.y)))
            } else {
                None
            }
        }
    }

    fn indentify_next_sibling(
        &self,
        current: SlotId,
        first_rect_bound: &Rect,
        new_size: Size,
    ) -> Option<(SlotId, Point)> {
        let current_rect = unsafe { self.allocated.get_unchecked(&current) };

        if let Some(first_child) = self.tree.get_first_child(current) {
            let first_child_rect = unsafe { self.allocated.get_unchecked(&first_child) };

            let mut curr = first_child;

            while let Some(next) = self.tree.get_next_sibling(curr) {
                let find = self.indentify_next_sibling(next, first_child_rect, new_size);
                if find.is_some() { return find }
                curr = next;
            }
        }

        let cond1 = new_size.width + current_rect.max_x() <= first_rect_bound.max_x();
        let cond2 = new_size.height <= current_rect.height;

        if cond1 && cond2 {
            Some((current, Point::new(current_rect.max_x(), current_rect.y)))
        } else {
            None
        }
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

    fn calculate_available_area(&self) -> f32 {
        let allocated = self.allocated.iter()
            .fold(0.0,|sum, (_, rect)| {
                sum + rect.area()
            });

        self.bound.area() - allocated
    }

    pub(crate) fn get_uv(&self, rect: Rect) -> Uv {
        let min_x = rect.x / self.bound.width;
        let min_y = rect.y / self.bound.height;
        let max_x = min_x + rect.width / self.bound.width;
        let max_y = min_y + rect.height / self.bound.height;

        Uv {
            min_x,
            min_y,
            max_x,
            max_y,
        }
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
