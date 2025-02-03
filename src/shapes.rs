use math::{tan, Matrix, Size, Vector2, Vector4};
use crate::context::{Cursor, LayoutCtx};
use crate::renderer::Buffer;
use crate::color::Rgb;
use crate::storage::cast_slice;
use crate::NodeId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    FilledTriangle,
    FilledRectangle,
    TexturedRectangle,
}

impl ShapeKind {
    pub fn is_triangle(&self) -> bool { matches!(self, Self::FilledTriangle) }
}

impl From<u32> for ShapeKind {
    fn from(num: u32) -> Self {
        match num {
            0 => Self::FilledTriangle,
            1 => Self::FilledRectangle,
            2 => Self::TexturedRectangle,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Indices<'a>(&'a [u32]);

impl std::ops::Deref for Indices<'_> {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl From<ShapeKind> for Indices<'_> {
    fn from(kind: ShapeKind) -> Self {
        match kind {
            ShapeKind::FilledTriangle => Self::triangle(),
            ShapeKind::FilledRectangle => Self::rectangle(),
            ShapeKind::TexturedRectangle => Self::rectangle(),
        }
    }
}

impl Indices<'_> {
    fn rectangle() -> Self {
        Self(&[0, 1, 2, 2, 3, 0])
    }

    fn triangle() -> Self {
        Self(&[4, 1, 2])
    }
}

// pub struct Vertices<'a>(&'a [Vector2<f32>]);

// #[repr(C)]
// struct Uniform {
//     kind: u32,
//     dimension: Size<f32>,
//     position: Vector2<f32>,
//     radius: f32,
//     transform: Matrix<Vector4<f32>, 4>,
// }

#[derive(Debug, Clone)]
pub struct Shape {
    pub dimensions: Size<u32>,
    pub color: Rgb<u8>,
    pub cached_color: Option<Rgb<u8>>,
    pub kind: u32,
    pub transform: Matrix<Vector4<f32>, 4>,
}

impl Shape {
    pub fn filled(color: Rgb<u8>, kind : ShapeKind, size: impl Into<Size<u32>>) -> Self {
        Self {
            dimensions: size.into(),
            color,
            cached_color: Some(color),
            kind: kind as u32,
            transform: Matrix::IDENTITY,
        }
    }

    pub fn textured(kind: ShapeKind) -> Self {
        Self {
            dimensions: Size::new(500, 500),
            color: Rgb::WHITE,
            cached_color: None,
            kind: kind as u32,
            transform: Matrix::IDENTITY,
        }
    }

    pub fn transform<F: FnMut(&mut Matrix<Vector4<f32>, 4>)>(&mut self, mut f: F) {
        f(&mut self.transform)
    }

    pub fn i_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Vec<u32>> {
        let kind = ShapeKind::from(self.kind);
        let indices = &Indices::from(kind);
        Buffer::i(device, cast_slice(indices), node_id.to_string())
    }

    pub fn u_buffer(&self, node_id: NodeId, device: &wgpu::Device) -> Buffer<Matrix<Vector4<f32>, 4>> {
        Buffer::u(device, cast_slice(self.transform.data()), node_id.to_string())
    }

    pub fn is_hovered(&self, cursor: &Cursor, center: Vector2<u32>) -> bool {
        let x = center.x as f32;
        let y = center.y as f32;

        let x_cursor = cursor.hover.pos.x;
        let y_cursor = cursor.hover.pos.y;

        let width = self.dimensions.width as f32 / 2.0;
        let height = self.dimensions.height as f32 / 2.0;

        let angled = if ShapeKind::from(self.kind).is_triangle() {
            let c_tangen = tan(x_cursor - x, y_cursor - y + height);
            let t_tangen = tan(width / 2.0, height);
            (t_tangen - c_tangen).is_sign_negative()
        } else { true };

        (y - height..y + height).contains(&y_cursor)
            && (x - width..x + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnOnce(&mut Rgb<u8>)>(&mut self, f: F) { f(&mut self.color) }

    pub fn revert_color(&mut self) -> bool {
        if let Some(cached) = self.cached_color {
            self.color = cached;
            true
        } else { false }
    }

    pub fn set_position(
        &mut self,
        cursor: &Cursor,
        node_id: NodeId,
        layout: &mut LayoutCtx
    ) {
        let delta = cursor.hover.pos - cursor.click.delta;
        if let Some(center) = layout.get_mut_position(node_id) {
            *center = delta.into();
        }
        let x = (delta.x / (self.dimensions.width as f32 / self.transform[0].x) - 0.5) * 2.0;
        let y = (0.5 - delta.y / (self.dimensions.height as f32 / self.transform[1].y)) * 2.0;
        self.transform(|mat| mat.translate(x, y));
    }
}

