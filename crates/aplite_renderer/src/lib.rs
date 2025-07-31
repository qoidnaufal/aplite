mod atlas;
mod buffer;
mod shader;
mod util;
mod element;
mod storage;
mod screen;
mod renderer;
mod mesh;

pub use renderer::{Renderer, Scene};
pub use element::{Element, Shape};
pub use mesh::Vertices;

#[derive(Debug)]
pub enum InitiationError {
    CreateSurfaceError,
    RequestAdapterError,
    RequestDeviceError,
}

#[derive(Debug)]
pub enum RenderError {
    TextureAcquiringFailed,
    ShouldResize,
    ShouldExit,
    TimeOut,
    PollError,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!{f, "{self:?}"}
    }
}

impl std::fmt::Display for InitiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RenderError {}
impl std::error::Error for InitiationError {}

impl From<wgpu::CreateSurfaceError> for InitiationError {
    fn from(_: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurfaceError
    }
}

impl From<wgpu::RequestAdapterError> for InitiationError {
    fn from(_: wgpu::RequestAdapterError) -> Self {
        Self::RequestAdapterError
    }
}

impl From<wgpu::RequestDeviceError> for InitiationError {
    fn from(_: wgpu::RequestDeviceError) -> Self {
        Self::RequestDeviceError
    }
}

impl From<wgpu::SurfaceError> for RenderError {
    fn from(value: wgpu::SurfaceError) -> Self {
        match value {
            wgpu::SurfaceError::Timeout => Self::TimeOut,
            wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => Self::ShouldResize,
            wgpu::SurfaceError::OutOfMemory => Self::ShouldExit,
            wgpu::SurfaceError::Other => Self::TextureAcquiringFailed,
        }
    }
}

impl From<wgpu::PollError> for RenderError {
    fn from(_: wgpu::PollError) -> Self {
        Self::PollError
    }
}
