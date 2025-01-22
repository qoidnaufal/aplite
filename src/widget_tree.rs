use std::collections::HashMap;
use math::{Size, Vector2};
use crate::renderer::Gfx;
use crate::color::Color;
use crate::view::{NodeId, View};
use crate::texture::{image_reader, ImageData, TextureData};
use crate::shapes::Shape;
use crate::error::Error;
use crate::callback::CALLBACKS;
use crate::app::CONTEXT;

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
pub struct WidgetTree {
    pub nodes: Vec<NodeId>,
    pub shapes: HashMap<NodeId, Shape>,
    has_changed: bool,
    last_changed_id: Option<NodeId>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            has_changed: false,
            last_changed_id: None,
        }
    }

    pub fn insert(&mut self, node: impl View) -> &mut Self {
        let id = node.id();
        let shape = node.shape();
        self.nodes.push(id);
        self.shapes.insert(id, shape);
        self
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed
    }

    pub fn invalidate_change(&mut self) {
        self.has_changed = false;
    }

    pub fn process_texture(
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
                    ImageData {
                        dimension: (1, 1).into(),
                        data: Color::from(shape.color).to_vec(),
                    }
                };
                let v = shape.v_buffer(device);
                let i = shape.i_buffer(device);
                let u = shape.u_buffer(device);
                let t = TextureData::new(
                    device,
                    queue,
                    bg_layout,
                    u,
                    image_data.dimension,
                    &image_data.data,
                    *node_id,
                );

                gfx.v_buffer.push(v);
                gfx.i_buffer.push(i);
                gfx.textures.push(t);
            }
        });
    }

    pub fn detect_hover(&self) {
        let hovered = self.shapes.iter().find(|(_, shape)| {
            shape.is_hovered()
        });
        if let Some((id, _)) = hovered {
            CONTEXT.with_borrow_mut(|ctx| {
                if let Some(click_id) = ctx.cursor.click.obj {
                    ctx.cursor.hover.obj = Some(click_id);
                } else {
                    ctx.cursor.hover.obj = Some(*id);
                }
            })
        } else {
            CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.obj = None)
        }
    }

    pub fn handle_hover(&mut self, queue: &wgpu::Queue, gfx: &Gfx) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let (Some(ref hover_id), Some(ref change_id), None) = (
            cursor.hover.obj, self.last_changed_id, cursor.click.obj
        ) { if hover_id == change_id { return; } }

        if let Some(ref change_id) = self.last_changed_id.take() {
            if cursor.hover.obj.is_some_and(|hover_id| hover_id != *change_id) || cursor.hover.obj.is_none() {
                let shape = self.shapes.get_mut(change_id).unwrap();
                if shape.revert_color() {
                    if let Some(texture) = gfx.textures.iter().find(|t| t.node_id == *change_id) {
                        texture.change_color(queue, shape.color);
                    }
                    self.has_changed = true;
                }
            }
        }
        if let Some(ref hover_id) = cursor.hover.obj {
            let shape = self.shapes.get_mut(hover_id).unwrap();
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                    on_hover(shape);
                }
                if cursor.is_dragging(*hover_id) {
                    if let Some(on_drag) = callbacks.on_drag.get_mut(hover_id) {
                        on_drag(shape);
                    }
                }
            });
            
            if let Some(texture) = gfx.textures.iter().find(|t| t.node_id == *hover_id) {
                texture.change_color(queue, shape.color);
            }
            self.has_changed = true;
            self.last_changed_id = Some(*hover_id);
        }
    }

    pub fn handle_click(&mut self, queue: &wgpu::Queue, gfx: &Gfx) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let Some(ref click_id) = cursor.click.obj {
            let shape = self.shapes.get_mut(click_id).unwrap();
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_click) = callbacks.on_click.get_mut(click_id) {
                    on_click(shape);
                }
            });
            if let Some(texture) = gfx.textures.iter().find(|t| t.node_id == *click_id) {
                texture.change_color(queue, shape.color);
            }
            self.has_changed = true;
            self.last_changed_id = Some(*click_id)
        }
    }

    pub fn recalculate_layout(&mut self, queue: &wgpu::Queue, gfx: &Gfx) {
        let ws = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let window_size: Size<f32> = ws.into();

        self.nodes.iter().for_each(|node_id| {
            let shape = self.shapes.get_mut(node_id).unwrap();
            let new_scale = Size::<f32>::from(shape.dimensions) / window_size / 2.0;
            let delta_scale = new_scale - Size::new(shape.transform[0].x, shape.transform[1].y);
            // let x = s.width - shape.transform[0].x;   // this might later be useful for flex style
            // let y = s.height - shape.transform[1].y;  // this might later be useful for flex style
            //
            // the transform means how "far" the shape from the left & top edge
            let x = shape.transform[3].x - delta_scale.width;
            let y = shape.transform[3].y - delta_scale.height;
            let new_translate = Vector2 { x, y };
            shape.set_transform(new_translate, new_scale);
            gfx.textures[node_id.0 as usize].u_buffer.update(queue, 0, shape.transform.as_slice());
        });
    }

    pub fn compute_layout(&mut self) {
        let window_size: Size<f32> = CONTEXT.with_borrow(|ctx| ctx.window_size.into());
        // this should be something like &mut LayoutCtx
        let mut used_space = Size::new(0, 0);

        self.nodes.iter().for_each(|id| {
            let shape = self.shapes.get_mut(id).unwrap();
            let scale = Size::<f32>::from(shape.dimensions) / window_size / 2.0; // div by 2.0 to set the center
            let used = Size::<f32>::from(used_space) / window_size;
            let x = (used.width + scale.width) - 1.0;   // -1.0 is the left edge of the screen coordinate
            let y = 1.0 - (used.height + scale.height); //  1.0 is the top  edge of the screen coordinate
            let translate = Vector2 { x, y };
            shape.set_transform(translate, scale);
            used_space.height += shape.dimensions.height;
        });
    }
}

