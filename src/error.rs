#[derive(Debug)]
pub enum Error {
    EventLoopCreation(winit::error::EventLoopError),
    SurfaceCreation(wgpu::CreateSurfaceError),
    SurfaceRendering(wgpu::SurfaceError),
    DeviceRequest(wgpu::RequestDeviceError),
    NoAdapterFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_kind = match self {
            Self::EventLoopCreation(err) => err.to_string(),
            Self::SurfaceCreation(err) => err.to_string(),
            Self::SurfaceRendering(err) => err.to_string(),
            Self::DeviceRequest(err) => err.to_string(),
            Self::NoAdapterFound => "No adapter found".to_string(),
        };

        write!(f, "{}", err_kind)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::EventLoopCreation(err) => err.source(),
            Error::SurfaceCreation(err) => err.source(),
            Error::SurfaceRendering(err) => err.source(),
            Error::DeviceRequest(err) => err.source(),
            Error::NoAdapterFound => None,
        }
    }
}

impl From<winit::error::EventLoopError> for Error {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoopCreation(value)
    }
}

impl From<wgpu::CreateSurfaceError> for Error {
    fn from(value: wgpu::CreateSurfaceError) -> Self {
        Self::SurfaceCreation(value)
    }
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        Self::DeviceRequest(value)
    }
}

impl From<wgpu::SurfaceError> for Error {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::SurfaceRendering(value)
    }
}
