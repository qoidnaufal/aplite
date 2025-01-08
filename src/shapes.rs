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

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    translate: Vector3<f32>,
    rotate: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translate: Vector3::new(),
            rotate: Vector3::new(),
        }
    }

    pub fn set_translate(&mut self, new_translate: Vector3<f32>) {
        self.translate = new_translate;
    }

    pub fn set_rotate(&mut self, new_rotate: Vector3<f32>) {
        self.rotate = new_rotate;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilledShape {
    FilledTriangle,
    FilledRectangle,
    FilledCircle,
}

#[derive(Debug, Clone, Copy)]
pub enum TexturedShape {
    TexturedTriangle,
    TexturedRectangle,
    TexturedCircle,
}

#[derive(Debug, Clone)]
pub struct FilledShapeData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

// originaly, every shape is rooted to the center of the screen where center is [0, 0]
// going top    -> [ 0,  y ],
// going left   -> [-x,  0 ],
// going bottom -> [ 0, -y ],
// going right  -> [ x,  0 ],
//
//
// a normal square with (width, height) would have
// top left     [x - width/2, y + height/2],
// bottom left  [x - width/2, y - height/2],
// bottom right [x + width/2, y - height/2],
// top right    [x + width/2, y + height/2],
// where (width, height) is normalized to window's inner_size

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub pos: Vector2<u32>,
    pub size: Size<u32>,
    pub color: Rgb<u8>,
    pub cached_color: Rgb<u8>,
    // pub transform: Transform,
    pub typ_ : FilledShape,
}

impl Shape {
    pub fn new(pos: Vector2<u32>, size: Size<u32>, color: Rgb<u8>, typ_ : FilledShape) -> Self {
        Self { pos, size, color, cached_color: color, /* transform: Transform::new(), */ typ_ }
    }

    pub fn filled(&self) -> FilledShapeData {
        match self.typ_ {
            FilledShape::FilledTriangle => self.filled_triangle(),
            FilledShape::FilledRectangle => self.filled_rectangle(),
            _ => self.filled_rectangle(),
        }
    }

    fn filled_triangle(&self) -> FilledShapeData {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let t = Vector3 { x: x_pos + x_center, y: y_pos, z: 0.0 };
        let l = Vector3 { x: x_pos, y: y_pos + height, z: 0.0 };
        let r = Vector3 { x: x_pos + width, y: y_pos + height, z: 0.0 };

        FilledShapeData {
            vertices: [
                Vertex { position: t, color: self.color.into() },
                Vertex { position: l, color: self.color.into() },
                Vertex { position: r, color: self.color.into() },
            ].to_vec(),
            indices: [0, 1, 2].to_vec()
        }
    }

    fn filled_rectangle(&self) -> FilledShapeData {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);

        // let tl = Vector3 { x: -1.0 * width/2, y:  1.0 * height/2, z: 0.0 }.transform();
        // let bl = Vector3 { x: -1.0 * width/2, y: -1.0 * height/2, z: 0.0 }.transform();
        // let br = Vector3 { x:  1.0 * width/2, y: -1.0 * height/2, z: 0.0 }.transform();
        // let tr = Vector3 { x:  1.0 * width/2, y:  1.0 * height/2, z: 0.0 }.transform();

        let tl = Vector3 { x: x_pos,         y: y_pos,          z: 0.0 };
        let bl = Vector3 { x: x_pos,         y: y_pos + height, z: 0.0 };
        let br = Vector3 { x: x_pos + width, y: y_pos + height, z: 0.0 };
        let tr = Vector3 { x: x_pos + width, y: y_pos,          z: 0.0 };

        FilledShapeData {
            vertices: [
                Vertex { position: tl, color: self.color.into() },
                Vertex { position: bl, color: self.color.into() },
                Vertex { position: br, color: self.color.into() },
                Vertex { position: tr, color: self.color.into() },
            ].to_vec(),
            indices: [0, 1, 2, 2, 3, 0].to_vec()
        }
    }

    fn textured_rectangle(&self) {}

    fn dimension(&self) -> Size<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);
        Size { width, height }
    }

    fn pos(&self) -> Vector2<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let x = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        Vector2 { x, y }
    }

    pub fn is_hovered(&self) -> bool {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, ctx.window_size));
        let x_cursor = ((cursor.hover.pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / window_size.height as f32)) * 2.0;

        let Size { width, height } = self.dimension();
        let Vector2 { x: x_pos, y: y_pos } = self.pos();

        let angled = if self.typ_ == FilledShape::FilledTriangle {
            let x_center = width / 2.0;
            let cursor_tan = tan(x_pos + x_center - x_cursor, y_pos - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        (y_pos + height..y_pos).contains(&y_cursor)
            && (x_pos..x_pos + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnMut(&mut Rgb<u8>)>(&mut self, mut f: F) {
        f(&mut self.color);
    }

    pub fn revert_color(&mut self) {
        self.color = self.cached_color;
    }

    pub fn set_position(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        let mut conv: Vector2<f32> = self.pos.into();
        let transform = (cursor.hover.pos - cursor.click.pos) * 2.0;
        conv += transform;
        self.pos = conv.into();

        CONTEXT.with_borrow_mut(|ctx| {
            ctx.cursor.click.pos = cursor.hover.pos;
        });
    }
}

