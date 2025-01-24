use std::path::PathBuf;

use math::{tan, Matrix, Size, Vector2, Vector3, Vector4};
use crate::renderer::Buffer;
use crate::storage::cast_slice;
use crate::color::Rgb;
use crate::app::CONTEXT;
use crate::NodeId;

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
#[derive(Clone, Copy)]
pub struct Transform {
    mat: Matrix<Vector4<f32>, 4>,
}

impl std::ops::Index<usize> for Transform {
    type Output = Vector4<f32>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.mat[index]
    }
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.mat)
    }
}

impl Transform {
    const IDENTITY: Self = Self { mat: Matrix::IDENTITIY };

    fn transform(&mut self, t: Vector2<f32>, s: Size<f32>) {
        self.mat.transform(t.x, t.y, s.width, s.height)
    }

    fn translate(&mut self, t: Vector2<f32>) {
        self.mat.translate(t.x, t.y);
    }

    // fn scale(&mut self, s: Size<f32>) {
    //     self.mat.scale(s.width, s.height);
    // }

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

impl ShapeKind {
    pub fn is_triangle(&self) -> bool {
        match self {
            ShapeKind::FilledTriangle => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh<'a> {
    pub vertices: &'a [Vertex],
    pub indices: &'a [u32],
}

impl From<ShapeKind> for Mesh<'_> {
    fn from(kind: ShapeKind) -> Self {
        match kind {
            ShapeKind::FilledTriangle => Self::triangle(),
            ShapeKind::FilledRectangle => Self::rectangle(),
            ShapeKind::TexturedRectangle => Self::rectangle(),
        }
    }
}

impl Mesh<'_> {
    fn rectangle() -> Self {
        Self {
            vertices: &[
                Vertex { position: Vector3 { x: -1.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 0.0 } },
                ],
            indices: &[0, 1, 2, 2, 3, 0],
        }
    }

    fn triangle() -> Self {
        Self {
            vertices: &[
                Vertex { position: Vector3 { x:  0.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 0.5, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
                ],
            indices: &[0, 1, 2],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub dimensions: Size<u32>,
    pub color: Rgb<u8>,
    pub cached_color: Option<Rgb<u8>>,
    pub src: Option<PathBuf>,
    pub kind: ShapeKind,
    pub transform: Transform,
}

impl Shape {
    pub fn filled(color: Rgb<u8>, kind : ShapeKind) -> Self {
        Self {
            dimensions: Size::new(500, 500),
            color,
            cached_color: Some(color),
            src: None,
            kind,
            transform: Transform::IDENTITY,
        }
    }

    pub fn textured(src: PathBuf, kind: ShapeKind) -> Self {
        Self {
            dimensions: Size::new(500, 500),
            color: Rgb::WHITE,
            cached_color: None,
            src: Some(src),
            kind,
            transform: Transform::IDENTITY,
        }
    }

    pub fn set_transform(&mut self, t: Vector2<f32>, s: Size<f32>) {
        self.transform.transform(t, s);
    }

    pub fn set_translate(&mut self, t: Vector2<f32>) {
        self.transform.translate(t);
    }

    // pub fn set_scale(&mut self, s: Size<f32>) {
    //     self.transform.scale(s);
    // }

    pub fn v_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Vertex> {
        let vertices = Mesh::from(self.kind).vertices;
        Buffer::new(device, wgpu::BufferUsages::VERTEX, cast_slice(vertices).unwrap(), node_id)
    }

    pub fn i_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Vec<u32>> {
        let indices = Mesh::from(self.kind).indices;
        Buffer::new(device, wgpu::BufferUsages::INDEX, cast_slice(indices).unwrap(), node_id)
    }

    pub fn u_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Transform> {
        Buffer::new(device, wgpu::BufferUsages::UNIFORM, self.transform.as_slice(), node_id)
    }

    fn dimension(&self) -> Size<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let width = self.dimensions.width as f32 / window_size.width as f32;
        let height = -(self.dimensions.height as f32 / window_size.height as f32);
        Size { width, height }
    }

    pub fn pos(&self) -> Vector2<f32> {
        let vertices = Mesh::from(self.kind).vertices;
        let x = (self.transform.mat * Vector4::from(vertices[1].position)).x;
        let y = (self.transform.mat * Vector4::from(vertices[0].position)).y;
        Vector2 { x, y }
    }

    pub fn is_hovered(&self) -> bool {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor.clone(), ctx.window_size));
        let x_cursor = ((cursor.hover.pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / window_size.height as f32)) * 2.0;

        let Size { width, height } = self.dimension();
        let Vector2 { x, y } = self.pos();

        let angled = if self.kind.is_triangle() {
            let x_center = width / 2.0;
            let cursor_tan = tan(x + x_center - x_cursor, y - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        (y + height..y).contains(&y_cursor)
            && (x..x + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnOnce(&mut Rgb<u8>)>(&mut self, f: F) {
        f(&mut self.color);
    }

    pub fn revert_color(&mut self) -> bool {
        if let Some(cached) = self.cached_color {
            self.color = cached;
            true
        } else { false }
    }

    pub fn set_position(&mut self) {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor.clone(), ctx.window_size));
        let ws = Size::<f32>::from(window_size) / 2.0;
        let x = cursor.hover.pos.x / ws.width - 1.0;
        let y = 1.0 - cursor.hover.pos.y / ws.height;
        let t = Vector2 { x, y };

        self.set_translate(t);
    }
}

