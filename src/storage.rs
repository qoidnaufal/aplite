use std::collections::HashMap;
use math::{Size, Vector2};
use crate::context::{LayoutCtx, MouseAction, CONTEXT};
use crate::renderer::{Gfx, Renderer};
use crate::view::NodeId;
use crate::texture::{image_reader, TextureData};
use crate::shapes::Shape;
use crate::error::Error;
use crate::callback::CALLBACKS;
use crate::{IntoView, View};

pub fn cast_slice<A: Sized, B: Sized>(p: &[A]) -> Result<&[B], Error> {
    if align_of::<B>() > align_of::<A>()
        && (p.as_ptr() as *const () as usize) % align_of::<B>() != 0 {
        return Err(Error::PointersHaveDifferentAlignmnet);
    }
    unsafe {
        let len = size_of_val::<[A]>(p) / size_of::<B>();
        Ok(core::slice::from_raw_parts(p.as_ptr() as *const B, len))
    }
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

    pub fn update(&mut self, renderer: &mut Renderer) {
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
        gfx: &mut Gfx,
    ) {
        self.nodes.iter().for_each(|node_id| {
            if let Some(shape) = self.shapes.get(node_id) {
                let image_data = if let Some(ref src) = shape.src {
                    image_reader(src)
                } else {
                    shape.color.into()
                };
                let v = shape.v_buffer(*node_id, device);
                let i = shape.i_buffer(*node_id, device);
                let u = shape.u_buffer(*node_id, device);
                let t = TextureData::new(
                    device,
                    queue,
                    bg_layout,
                    u,
                    image_data,
                );

                gfx.v_buffer.insert(*node_id, v);
                gfx.i_buffer.insert(*node_id, i);
                gfx.textures.insert(*node_id, t);
            }
        });
    }

    pub fn detect_hover(&self) {
        let hovered = self.shapes.iter().filter_map(|(id, shape)| {
            let pos = self.layout.get_position(id).copied().unwrap();
            if shape.is_hovered(pos) {
                Some(id)
            } else { None }
        }).min();
        if let Some(id) = hovered {
            CONTEXT.with_borrow_mut(|ctx| {
                if ctx.cursor.click.obj.is_none() {
                    ctx.cursor.hover.prev = ctx.cursor.hover.curr;
                    ctx.cursor.hover.curr = Some(*id);
                }
            })
        } else {
            CONTEXT.with_borrow_mut(|ctx| {
                ctx.cursor.hover.prev = ctx.cursor.hover.curr.take();
            });
        }
    }

    pub fn handle_hover(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(ref prev_id) = CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.prev.take()) {
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
                        shape.set_position();
                        self.layout.insert_pos(*hover_id, shape.pos());
                    }
                }
            });
            self.pending_update.push(*hover_id);
        }
    }

    pub fn handle_click(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
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

    pub fn layout(&mut self) {
        let ws: Size<f32> = CONTEXT.with_borrow(|ctx| ctx.window_size.into());

        self.nodes.iter().for_each(|node_id| {
            let shape = self.shapes.get_mut(node_id).unwrap();
            shape.scale();
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

