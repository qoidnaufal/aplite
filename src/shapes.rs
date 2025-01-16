use crate::{
    app::CONTEXT, color::{Color, Rgb, Rgba}, layout::cast_slice, texture::TextureData
};
use math::{tan, Matrix, Size, Vector2, Vector3};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.uv == other.uv
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    mat: Matrix<Vector3<f32>, 3>,
}

impl Transform {
    pub fn new(t: Vector2<f32>, s: Size<f32>) -> Self {
        Self {
            mat: Matrix::transform(t.x, t.y, s.width, s.height)
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        cast_slice(self.mat.data()).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    FilledTriangle,
    FilledRectangle,
    TexturedRectangle,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl From<ShapeKind> for Mesh {
    fn from(kind: ShapeKind) -> Self {
        match kind {
            ShapeKind::FilledTriangle => Self::triangle(),
            ShapeKind::FilledRectangle => Self::rectangle(),
            ShapeKind::TexturedRectangle => Self::rectangle(),
        }
    }
}

impl Mesh {
    fn rectangle() -> Self {
        Self {
            vertices: [
                Vertex { position: Vector3 { x: -1.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 0.0 } },
                ].to_vec(),
            indices: [0, 1, 2, 2, 3, 0].to_vec(),
        }
    }

    fn triangle() -> Self {
        Self {
            vertices: [
                Vertex { position: Vector3 { x:  0.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 0.5, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
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
    pub size: Size<u32>,
    pub cached_color: Option<Rgb<u8>>,
    pub mesh: Mesh,
    pub uv_size: Size<u32>,
    pub uv_data: Color<Rgba<u8>, u8>,
}

impl Shape {
    pub fn filled(color: Rgb<u8>, kind : ShapeKind) -> Self {
        Self {
            mesh: kind.into(),
            size: Size::new(500, 500),
            cached_color: Some(color),
            uv_size: (1, 1).into(),
            uv_data: color.into(),
        }
    }

    pub fn textured(uv_size: Size<u32>, texture_data: &[u8], kind: ShapeKind) -> Self {
        Self {
            mesh: kind.into(),
            size: Size::new(500, 500),
            cached_color: None,
            uv_size,
            uv_data: texture_data.into(),
        }
    }

    pub fn transform(&mut self, t: Vector2<f32>, s: Size<f32>) {
        self.mesh.vertices.iter_mut().for_each(|vert| {
            vert.position = Matrix::transform(t.x, t.y, s.width, s.height) * vert.position;
        });
    }

    pub fn _create_texture(
        &self,
        device: &wgpu::Device,
        bg_layout: &wgpu::BindGroupLayout,
        size: Size<u32>,
    ) -> TextureData {
        TextureData::new(device, bg_layout, size)
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.mesh.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.mesh.indices
    }

    // for now, i think the dimension will always be constant due to scaling transform
    // but still, i need better calculation later
    fn dimension(&self) -> Size<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let width = self.size.width as f32 / window_size.width as f32;
        let height = -(self.size.height as f32 / window_size.height as f32);
        Size { width, height }
    }

    fn pos(&self) -> Vector2<f32> {
        Vector2 {
            x: self.mesh.vertices[1].position.x,
            y: self.mesh.vertices[0].position.y,
        }
    }

    pub fn is_hovered(&self) -> bool {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, ctx.window_size));
        let x_cursor = ((cursor.hover.pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / window_size.height as f32)) * 2.0;

        let Size { width, height } = self.dimension();
        let Vector2 { x, y } = self.pos();

        let angled = if self.mesh.indices.len() == 3 {
            let x_center = width / 2.0;
            let cursor_tan = tan(x + x_center - x_cursor, y - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        (y + height..y).contains(&y_cursor)
            && (x..x + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnOnce(&mut Color<Rgba<u8>, u8>)>(&mut self, f: F) {
        f(&mut self.uv_data);
    }

    pub fn revert_color(&mut self) {
        if let Some(ref c) = self.cached_color {
            self.uv_data = c.clone().into();
        }
    }

    pub fn set_position(&mut self) {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, Size::<f32>::from(ctx.window_size)));
        let t = (cursor.hover.pos - cursor.click.pos) * 2.0;
        self.transform(
            Vector2 { x: t.x / window_size.width, y: -t.y / window_size.height },
            Size { width: 1.0, height: 1.0 }
        );

        CONTEXT.with_borrow_mut(|ctx| {
            ctx.cursor.click.pos = cursor.hover.pos;
        });
    }
}

