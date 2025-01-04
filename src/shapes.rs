use crate::{
    app::CONTEXT, color::Rgb, types::{tan, Size, Vector2, Vector3}
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
pub struct ShapeData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub pos: Vector2<u32>,
    pub size: Size<u32>,
    pub color: Rgb<u8>,
}

impl Shape {
    pub fn new(pos: Vector2<u32>, size: Size<u32>, color: Rgb<u8>) -> Self {
        Self { pos, size, color }
    }

    pub fn triangle(&self) -> ShapeData {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let t = Vector3 { x: x_pos + x_center, y: y_pos, z: 0.0 };
        let l = Vector3 { x: x_pos, y: y_pos + height, z: 0.0 };
        let r = Vector3 { x: x_pos + width, y: y_pos + height, z: 0.0 };

        ShapeData {
            vertices: [
                Vertex { position: t, color: self.color.into() },
                Vertex { position: l, color: self.color.into() },
                Vertex { position: r, color: self.color.into() },
            ].to_vec(),
            indices: [0, 1, 2].to_vec()
        }
    }

    pub fn rectangle(&self) -> ShapeData {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);

        let tl = Vector3 { x: x_pos, y: y_pos, z: 0.0 };
        let bl = Vector3 { x: x_pos, y: y_pos + height, z: 0.0 };
        let br = Vector3 { x: x_pos + width, y: y_pos + height, z: 0.0 };
        let tr = Vector3 { x: x_pos + width, y: y_pos, z: 0.0 };

        ShapeData {
            vertices: [
                Vertex { position: tl, color: self.color.into() },
                Vertex { position: bl, color: self.color.into() },
                Vertex { position: br, color: self.color.into() },
                Vertex { position: tr, color: self.color.into() },
            ].to_vec(),
            indices: [0, 1, 2, 2, 3, 0].to_vec()
        }
    }

    pub fn dimension(&self) -> Size<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);
        Size { width, height }
    }

    pub fn pos(&self) -> Vector2<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let x = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        Vector2 { x, y }
    }

    pub fn is_hovered(&self, indices_len: usize) -> bool {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, ctx.window_size));
        let x_cursor = ((cursor.hover.pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / window_size.height as f32)) * 2.0;

        let Size { width, height } = self.dimension();
        let Vector2 { x: x_pos, y: y_pos } = self.pos();

        let angled = if indices_len == 3 {
            let x_center = width / 2.0;
            let cursor_tan = tan(x_pos + x_center - x_cursor, y_pos - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        // if cursor.click.obj.is_some_and(|click_id| click_id != id) { return false; }

        (y_pos + height..y_pos).contains(&y_cursor)
            && (x_pos..x_pos + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnMut(&mut Rgb<u8>)>(&mut self, mut f: F) {
        f(&mut self.color);
    }

    pub fn set_position(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        // println!("hovered");
        let delta_x = cursor.hover.pos.x - cursor.click.pos.x;
        let delta_y = cursor.hover.pos.y - cursor.click.pos.y;
        let transform = Vector2 { x: delta_x * 2.0, y: delta_y * 2.0 };

        let mut conv: Vector2<f32> = self.pos.into();
        conv.translation(transform);
        self.pos = conv.into();

        CONTEXT.with_borrow_mut(|ctx| {
            ctx.cursor.click.pos = cursor.hover.pos;
        });
    }
}

