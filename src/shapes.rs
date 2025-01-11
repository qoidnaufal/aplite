use crate::{
    app::CONTEXT,
    color::{Color, Rgb, Rgba}, pipeline::Texture,
};
use math::{tan, Matrix, Size, Vector2, Vector3};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector2<f32>,
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
                    format: wgpu::VertexFormat::Float32x2,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    FilledTriangle,
    FilledRectangle,
    TexturedRectangle,
}

#[derive(Debug, Clone)]
pub struct ShapeData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl From<ShapeType> for ShapeData {
    fn from(typ_: ShapeType) -> Self {
        match typ_ {
            ShapeType::FilledTriangle => Self::triangle(),
            ShapeType::FilledRectangle => Self::rectangle(),
            ShapeType::TexturedRectangle => Self::rectangle(),
        }
    }
}

impl ShapeData {
    fn rectangle() -> Self {
        Self {
            vertices: [
                Vertex { position: Vector3 { x: -1.0, y:  1.0, z: 1.0 }, color: Vector2 { x: 0.0, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, color: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, color: Vector2 { x: 1.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y:  1.0, z: 1.0 }, color: Vector2 { x: 1.0, y: 0.0 } },
                ].to_vec(),
            indices: [0, 1, 2, 2, 3, 0].to_vec(),
        }
    }

    fn triangle() -> Self {
        Self {
            vertices: [
                Vertex { position: Vector3 { x:  0.0, y:  1.0, z: 1.0 }, color: Vector2 { x: 0.5, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, color: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, color: Vector2 { x: 1.0, y: 1.0 } },
                ].to_vec(),
            indices: [0, 1, 2].to_vec(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct Shape {
    pub shape_data: ShapeData,
    pub size: Size<u32>,
    pub uv_size: Size<u32>,
    pub uv_data: Color<Rgba<u8>, u8>,
    pub cached_color: Option<Color<Rgba<u8>, u8>>,
    pub transform: Matrix<Vector3<f32>, 3>,
}

impl Shape {
    pub fn filled(color: Rgb<u8>, typ_ : ShapeType) -> Self {
        Self {
            shape_data: typ_.into(),
            size: Size::new(500, 500),
            uv_size: (1, 1).into(),
            uv_data: color.into(),
            cached_color: Some(color.into()),
            transform: Matrix::IDENTITIY,
        }
    }

    pub fn textured(uv_size: Size<u32>, texture_data: &[u8], typ_: ShapeType) -> Self {
        Self {
            shape_data: typ_.into(),
            size: Size::new(500, 500),
            uv_size,
            uv_data: texture_data.into(),
            cached_color: None,
            transform: Matrix::IDENTITIY,
        }
    }

    pub fn transform(&mut self) {
        self.shape_data.vertices.iter_mut().for_each(|vert| {
            vert.position = self.transform * vert.position;
        });
        self.transform = Matrix::IDENTITIY;
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.transform.translation(tx, ty);
    }

    pub fn scale(&mut self, scale: Size<f32>) {
        self.transform.scale(scale.width, scale.height);
    }

    pub fn process_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
        Texture::new(device, queue, self.uv_size, &self.uv_data)
    }

    // for now, i think the dimension will always be constant due to scaling transform
    // but still, i need better calculation later
    fn dimension(&self) -> Size<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);
        Size { width, height }
    }

    pub fn is_hovered(&self) -> bool {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, ctx.window_size));
        let x_cursor = ((cursor.hover.pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / window_size.height as f32)) * 2.0;

        let Size { width, height } = self.dimension();
        let  x_pos = self.shape_data.vertices[1].position.x;
        let  y_pos = self.shape_data.vertices[0].position.y;

        let angled = if self.shape_data.indices.len() == 3 {
            let x_center = width / 2.0;
            let cursor_tan = tan(x_pos + x_center - x_cursor, y_pos - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        (y_pos + height..y_pos).contains(&y_cursor)
            && (x_pos..x_pos + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnMut(&mut Color<Rgba<u8>, u8>)>(&mut self, mut f: F) {
        f(&mut self.uv_data);
    }

    pub fn revert_color(&mut self) {
        if let Some(ref c) = self.cached_color {
            self.uv_data = c.clone();
        }
    }

    pub fn set_position(&mut self) {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, Size::<f32>::from(ctx.window_size)));
        let t = (cursor.hover.pos - cursor.click.pos) * 2.0;
        self.translate(t.x / window_size.width, -t.y / window_size.height);
        self.transform();

        CONTEXT.with_borrow_mut(|ctx| {
            ctx.cursor.click.pos = cursor.hover.pos;
        });
    }
}

