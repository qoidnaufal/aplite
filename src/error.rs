#[derive(Debug)]
pub enum Error {
    EventLoopError(winit::error::EventLoopError),
    CreateSurfaceError(wgpu::CreateSurfaceError),
    SurfaceError(wgpu::SurfaceError),
    RequestDeviceError(wgpu::RequestDeviceError),
    NoAdapterFound,
    PointersHaveDifferentAlignmnet,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_kind = match self {
            Self::EventLoopError(err) => err.to_string(),
            Self::CreateSurfaceError(err) => err.to_string(),
            Self::SurfaceError(err) => err.to_string(),
            Self::RequestDeviceError(err) => err.to_string(),
            Self::NoAdapterFound => "No adapter found".to_string(),
            Self::PointersHaveDifferentAlignmnet => "Alignment doesn't match".to_string(),
        };

        write!(f, "{}", err_kind)
    }
}

impl std::error::Error for Error {}

impl From<winit::error::EventLoopError> for Error {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoopError(value)
    }
}

impl From<wgpu::CreateSurfaceError> for Error {
    fn from(value: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurfaceError(value)
    }
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        Self::RequestDeviceError(value)
    }
}

impl From<wgpu::SurfaceError> for Error {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::SurfaceError(value)
    }
}
