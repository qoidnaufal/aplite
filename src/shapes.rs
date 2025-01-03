use crate::{
    app::CONTEXT,
    color::Rgb,
    types::{tan, Size, Vector2, Vector3}
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Rgb<f32>,
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.color == other.color
    }
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Shape {
    pub fn triangle(pos: Vector3<u32>, size: Size<u32>, color: Rgb<u8>) -> Self {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (pos.y as f32 / window_size.height as f32);
        
        let width = size.width as f32 / window_size.width as f32;
        let height = -(size.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let t = Vector3 { x: x_pos + x_center, y: y_pos, z: pos.z as _ };
        let l = Vector3 { x: x_pos, y: y_pos + height, z: pos.z as _ };
        let r = Vector3 { x: x_pos + width, y: y_pos + height, z: pos.z as _ };

        Self {
            vertices: [
                Vertex { position: t, color: color.into() },
                Vertex { position: l, color: color.into() },
                Vertex { position: r, color: color.into() },
            ].to_vec(),
            indices: [0, 1, 2].to_vec()
        }
    }

    pub fn rectangle(pos: Vector3<u32>, size: Size<u32>, color: Rgb<u8>) -> Self {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (pos.y as f32 / window_size.height as f32);
        
        let width = size.width as f32 / window_size.width as f32;
        let height = -(size.height as f32 / window_size.height as f32);

        let tl = Vector3 { x: x_pos, y: y_pos, z: pos.z as _ };
        let bl = Vector3 { x: x_pos, y: y_pos + height, z: pos.z as _ };
        let br = Vector3 { x: x_pos + width, y: y_pos + height, z: pos.z as _ };
        let tr = Vector3 { x: x_pos + width, y: y_pos, z: pos.z as _ };

        Self {
            vertices: [
                Vertex { position: tl, color: color.into() },
                Vertex { position: bl, color: color.into() },
                Vertex { position: br, color: color.into() },
                Vertex { position: tr, color: color.into() },
            ].to_vec(),
            indices: [0, 1, 2, 2, 3, 0].to_vec()
        }
    }

    pub fn dimension(&self) -> Size<f32> {
        let width = self.vertices[2].position.x - self.vertices[1].position.x;
        let height = - (self.vertices[0].position.y - self.vertices[1].position.y);
        Size { width, height }
    }

    pub fn pos(&self) -> Vector2<f32> {
        let x = self.vertices[1].position.x;
        let y = self.vertices[0].position.y;
        Vector2 { x, y }
    }

    pub fn is_hovered(&self) -> bool {
        let (x_cursor, y_cursor) = CONTEXT.with_borrow(|ctx| {
            let x_cursor = ((ctx.cursor.position.x / ctx.window_size.width as f32) - 0.5) * 2.0;
            let y_cursor = (0.5 - (ctx.cursor.position.y / ctx.window_size.height as f32)) * 2.0;
            (x_cursor, y_cursor)
        });

        let dimension = self.dimension();
        let width = dimension.width;
        let height = dimension.height;

        let pos = self.pos();
        let x_pos = pos.x;
        let y_pos = pos.y;

        let hover = if self.indices.len() == 3 {
            let x_center = width / 2.0;
            let cursor_tan = tan(x_pos + x_center - x_cursor, y_pos - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        (y_pos + height..y_pos).contains(&y_cursor)
            && (x_pos..x_pos + width).contains(&x_cursor)
            && hover
    }

    pub fn handle_click(&mut self) {
        if self.is_hovered() {
            match CONTEXT.with_borrow(|ctx| ctx.cursor.state.action) {
                winit::event::ElementState::Pressed => {
                    // println!("clicked");
                    self.set_color(|c| {
                        *c = Rgb { r: 0, g: 255, b: 0 };
                    });
                    CONTEXT.with_borrow_mut(|ctx| {
                        ctx.cursor.click.x = ctx.cursor.position.x;
                        ctx.cursor.click.y = ctx.cursor.position.y;
                    });
                },
                winit::event::ElementState::Released => {
                    self.set_color(|c| {
                        *c = Rgb { r: 0, g: 0, b: 255 };
                    });
                },
            }
        }
    }

    pub fn set_color<F: FnMut(&mut Rgb<u8>)>(&mut self, mut f: F) {
        self.vertices.iter_mut().for_each(|vert| {
            let mut new_color: Rgb<u8> = vert.color.into();
            f(&mut new_color);
            vert.color = new_color.into();
        });
    }

    pub fn set_position(&mut self) {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, ctx.window_size));
        if self.is_hovered() {
            // println!("hovered");
            match cursor.state.action {
                winit::event::ElementState::Pressed => {
                    self.set_color(|c| {
                        *c = Rgb { r: 0, g: 255, b: 0 };
                    });
                    let delta_x = (cursor.position.x - cursor.click.x) / window_size.width as f32;
                    let delta_y = (cursor.position.y - cursor.click.y) / window_size.height as f32;
                    let transform = Vector2 { x: delta_x * 2.0, y: delta_y * -2.0 };

                    self.vertices.iter_mut().for_each(|vert| {
                        vert.position.translation(transform);

                        CONTEXT.with_borrow_mut(|ctx| {
                            ctx.cursor.click.x = cursor.position.x;
                            ctx.cursor.click.y = cursor.position.y;
                        });
                    });
                },
                winit::event::ElementState::Released => {
                    self.set_color(|c| {
                        *c = Rgb { r: 0, g: 0, b: 255 };
                    });
                },
            }
        } else {
            self.set_color(|c| {
                *c = Rgb { r: 255, g: 0, b: 0 };
            });
        }
    }
}

