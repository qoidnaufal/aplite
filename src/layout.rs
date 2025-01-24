use math::{Matrix, Size, Vector4};

pub struct LayoutCtx {
    size: Size<u32>,
    transform: Matrix<Vector4<f32>, 4>,
}
