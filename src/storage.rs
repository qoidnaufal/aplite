use std::collections::HashMap;
use math::{Size, Vector2};
use crate::context::{Cursor, LayoutCtx, MouseAction};
use crate::renderer::{Gfx, Renderer};
use crate::view::NodeId;
use crate::shapes::Shape;
use crate::callback::CALLBACKS;
use crate::{IntoView, View};

pub fn cast_slice<SRC: Sized, DST: Sized>(src: &[SRC]) -> &[DST] {
    let len = size_of_val::<[SRC]>(src) / size_of::<DST>();
    unsafe { core::slice::from_raw_parts(src.as_ptr() as *const DST, len) }
}

#[derive(Debug)]
pub struct WidgetStorage {
    pub nodes: Vec<NodeId>,
    pub shapes: HashMap<NodeId, Shape>,
    pub layout: LayoutCtx,
    pending_update: Vec<NodeId>,
}

impl WidgetStorage {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            layout: LayoutCtx::new(),
            pending_update: Vec::new(),
        }
    }

    pub fn insert(&mut self, node: impl IntoView) -> &mut Self {
        let node = node.into_view();
        node.layout(&mut self.layout);
        node.insert_into(self);
        self
    }

    pub fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub fn submit_update(&mut self, renderer: &mut Renderer) {
        while let Some(ref change_id) = self.pending_update.pop() {
            let shape = self.shapes.get(change_id).unwrap();
            renderer.update(change_id, shape);
        }
    }

    pub fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
        scenes: &mut HashMap<NodeId, Gfx>,
    ) {
        self.nodes.iter().for_each(|node_id| {
            if let Some(shape) = self.shapes.get(node_id) {
                let gfx = Gfx::new(device, queue, bg_layout, shape, *node_id);
                scenes.insert(*node_id, gfx);
            }
        });
    }

    pub fn detect_hover(&self, cursor: &mut Cursor, size: Size<u32>) {
        // let start = std::time::Instant::now();
        let hovered = self.shapes.iter().filter_map(|(id, shape)| {
            let pos = self.layout.get_position(id).copied().unwrap();
            if shape.is_hovered(cursor, pos, size) {
                Some(id)
            } else { None }
        }).min();
        // eprintln!("{:?}", start.elapsed());
        if let Some(id) = hovered {
            if cursor.click.obj.is_none() {
                cursor.hover.prev = cursor.hover.curr;
                cursor.hover.curr = Some(*id);
            }
        } else {
            cursor.hover.prev = cursor.hover.curr.take();
        }
    }

    pub fn handle_hover(&mut self, cursor: &mut Cursor, size: Size<u32>) {
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(ref prev_id) = cursor.hover.prev.take() {
            let shape = self.shapes.get_mut(prev_id).unwrap();
            if shape.revert_color() {
                self.pending_update.push(*prev_id);
            }
        }
        if let Some(ref hover_id) = cursor.hover.curr {
            let shape = self.shapes.get_mut(hover_id).unwrap();
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                    on_hover(shape);
                }
                if cursor.is_dragging(*hover_id) {
                    if let Some(on_drag) = callbacks.on_drag.get_mut(hover_id) {
                        on_drag(shape);
                        shape.set_position(cursor, size);
                        self.layout.insert_pos(*hover_id, shape.pos(size));
                    }
                }
            });
            self.pending_update.push(*hover_id);
        }
    }

    pub fn handle_click(&mut self, cursor: &Cursor) {
        if let Some(ref click_id) = cursor.click.obj {
            let shape = self.shapes.get_mut(click_id).unwrap();
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_click) = callbacks.on_click.get_mut(click_id) {
                    on_click(shape);
                    self.pending_update.push(*click_id);
                }
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let shape = self.shapes.get_mut(hover_id).unwrap();
                CALLBACKS.with_borrow_mut(|callbacks| {
                    if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                        on_hover(shape);
                        self.pending_update.push(*hover_id);
                    }
                });
            }
        }
    }

    pub fn layout(&mut self, size: Size<u32>) {
        let ws: Size<f32> = size.into();

        self.nodes.iter().for_each(|node_id| {
            let shape = self.shapes.get_mut(node_id).unwrap();
            shape.scale(size);
            let center: Vector2<f32> = self
                .layout
                .get_position(node_id)
                .copied()
                .unwrap()
                .into();
            let translate = Vector2 {
                x: (center.x / ws.width - 0.5) * 2.0,
                y: (0.5 - center.y / ws.height) * 2.0,
            };
            shape.set_translate(translate);
            self.pending_update.push(*node_id);
        });
    }
}

