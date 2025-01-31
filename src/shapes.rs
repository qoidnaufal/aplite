use std::path::PathBuf;

use math::{tan, Matrix, Size, Vector2, Vector4};
use crate::context::Cursor;
use crate::renderer::Buffer;
use crate::color::Rgb;
use crate::storage::cast_slice;
use crate::texture::image_reader;
use crate::{Color, NodeId, Rgba};

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct Vertex {
//     pub position: Vector2<f32>,
//     pub uv: Vector2<f32>,
// }

// impl Vertex {
//     pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         wgpu::VertexBufferLayout {
//             array_stride: size_of::<Self>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     format: wgpu::VertexFormat::Float32x2,
//                     offset: 0,
//                     shader_location: 0,
//                 },
//                 wgpu::VertexAttribute {
//                     format: wgpu::VertexFormat::Float32x2,
//                     offset: size_of::<Vector2<f32>>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                 },
//             ],
//         }
//     }
// }

// impl PartialEq for Vertex {
//     fn eq(&self, other: &Self) -> bool {
//         self.position == other.position
//             && self.uv == other.uv
//     }
// }

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    mat: Matrix<Vector4<f32>, 4>,
}

impl std::ops::Index<usize> for Transform {
    type Output = Vector4<f32>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.mat[index]
    }
}

impl Transform {
    const IDENTITY: Self = Self { mat: Matrix::IDENTITIY };

    pub fn set<F: FnMut(&mut Matrix<Vector4<f32>, 4>)>(&mut self, mut f: F) {
        f(&mut self.mat)
    }

    pub fn as_slice(&self) -> &[u8] {
        cast_slice(self.mat.data())
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
        matches!(self, Self::FilledTriangle)
    }
}

#[derive(Debug, Clone)]
pub struct Mesh<'a> {
    // pub vertices: &'a [Vertex],
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
            // vertices: &[
            //     Vertex { position: Vector2 { x: -1.0, y:  1.0 }, uv: Vector2 { x: 0.0, y: 0.0 } },
            //     Vertex { position: Vector2 { x: -1.0, y: -1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
            //     Vertex { position: Vector2 { x:  1.0, y: -1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
            //     Vertex { position: Vector2 { x:  1.0, y:  1.0 }, uv: Vector2 { x: 1.0, y: 0.0 } },
            //     ],
            indices: &[0, 1, 2, 2, 3, 0],
        }
    }

    fn triangle() -> Self {
        Self {
            // vertices: &[
            //     Vertex { position: Vector2 { x:  0.0, y:  1.0 }, uv: Vector2 { x: 0.5, y: 0.0 } },
            //     Vertex { position: Vector2 { x: -1.0, y: -1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
            //     Vertex { position: Vector2 { x:  1.0, y: -1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
            //     ],
            indices: &[4, 1, 2],
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
    pub fn filled(color: Rgb<u8>, kind : ShapeKind, size: impl Into<Size<u32>>) -> Self {
        Self {
            dimensions: size.into(),
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

    pub fn image_data(&self) -> Color<Rgba<u8>, u8> {
        if let Some(ref src) = self.src {
            image_reader(src)
        } else {
            self.color.into()
        }
    }

    pub fn set_translate(&mut self, t: Vector2<f32>) {
        self.transform.set(|mat| mat.translate(t.x, t.y));
    }

    pub fn scale(&mut self, size: Size<u32>) {
        let ws: Size<f32> = size.into();
        let s = Size::<f32>::from(self.dimensions) / ws;
        self.transform.set(|mat| mat.scale(s.width, s.height));
    }

    // pub fn v_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Vertex> {
    //     let vertices = Mesh::from(self.kind).vertices;
    //     Buffer::v(device, cast_slice(vertices), node_id)
    // }

    pub fn i_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Vec<u32>> {
        let indices = Mesh::from(self.kind).indices;
        Buffer::i(device, cast_slice(indices), node_id)
    }

    pub fn u_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Transform> {
        Buffer::u(device, self.transform.as_slice(), node_id)
    }

    pub fn pos(&self, size: Size<u32>) -> Vector2<u32> {
        let ws: Size<f32> = size.into();
        let x = (self.transform[3].x / 2.0 + 0.5) * ws.width;
        let y = (0.5 - self.transform[3].y / 2.0) * ws.height;
        Vector2::new(x as u32, y as u32)
    }

    pub fn is_hovered(&self, cursor: &Cursor, center: Vector2<u32>, size: Size<u32>) -> bool {
        let ws: Size<f32> = size.into();
        let x_cursor = ((cursor.hover.pos.x / ws.width) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / ws.height)) * 2.0;

        let x = (center.x as f32 / ws.width - 0.5) * 2.0;
        let y = (0.5 - center.y as f32 / ws.height) * 2.0;
        let Size { width, height } = Size::<f32>::from(self.dimensions) / ws;

        let angled = if self.kind.is_triangle() {
            let c_tangen = tan(x - x_cursor, y + height - y_cursor);
            let t_tangen = tan(width / 2.0, height);
            (t_tangen - c_tangen).is_sign_negative()
        } else { true };

        (y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
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

    pub fn set_position(&mut self, cursor: &Cursor, size: Size<u32>) {
        let ws = Size::<f32>::from(size);
        let x = (cursor.hover.pos.x / ws.width - 0.5) * 2.0;
        let y = (0.5 - cursor.hover.pos.y / ws.height) * 2.0;
        let t = Vector2 { x, y };

        self.set_translate(t);
    }
}

