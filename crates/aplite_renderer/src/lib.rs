mod atlas;
mod gpu;
mod buffer;
mod shader;
mod util;
mod element;
mod storage;
mod screen;
mod renderer;
mod mesh;

pub use renderer::Renderer;
pub use element::{Element, Shape};
pub use mesh::Vertices;
pub use gpu::{GpuDevice, GpuSurface};

#[derive(Debug)]
pub enum RendererError {
    CreateSurfaceError,
    RequestAdapterError,
    RequestDeviceError,
    TextureAcquiringFailed,
    ShouldResize,
    ShouldExit,
    TimeOut,
    PollError,
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!{f, "self:?"}
    }
}

impl std::error::Error for RendererError {}

impl From<wgpu::CreateSurfaceError> for RendererError {
    fn from(_: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurfaceError
    }
}

impl From<wgpu::RequestAdapterError> for RendererError {
    fn from(_: wgpu::RequestAdapterError) -> Self {
        Self::RequestAdapterError
    }
}

impl From<wgpu::RequestDeviceError> for RendererError {
    fn from(_: wgpu::RequestDeviceError) -> Self {
        Self::RequestDeviceError
    }
}

impl From<wgpu::SurfaceError>  for RendererError {
    fn from(value: wgpu::SurfaceError) -> Self {
        match value {
            wgpu::SurfaceError::Timeout => Self::TimeOut,
            wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => Self::ShouldResize,
            wgpu::SurfaceError::OutOfMemory => Self::ShouldExit,
            wgpu::SurfaceError::Other => Self::TextureAcquiringFailed,
        }
    }
}

impl From<wgpu::PollError> for RendererError {
    fn from(_: wgpu::PollError) -> Self {
        Self::PollError
    }
}
