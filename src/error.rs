#[derive(Debug)]
pub enum ApliteError {
    EventLoopCreationFailed(winit::error::EventLoopError),
    SurfaceCreationFailed(wgpu::CreateSurfaceError),
    DeviceRequestFailed(wgpu::RequestDeviceError),
    AdapterRequestFailed(wgpu::RequestAdapterError),
    WindowCreationFailed(winit::error::OsError),
}

impl std::fmt::Display for ApliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_kind = match self {
            Self::EventLoopCreationFailed(err) => err.to_string(),
            Self::SurfaceCreationFailed(err) => err.to_string(),
            Self::DeviceRequestFailed(err) => err.to_string(),
            Self::AdapterRequestFailed(err) => err.to_string(),
            Self::WindowCreationFailed(err) => err.to_string(),
        };

        write!(f, "{}", err_kind)
    }
}

impl std::error::Error for ApliteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EventLoopCreationFailed(err) => err.source(),
            Self::SurfaceCreationFailed(err) => err.source(),
            Self::DeviceRequestFailed(err) => err.source(),
            Self::AdapterRequestFailed(err) => err.source(),
            Self::WindowCreationFailed(err) => err.source(),
        }
    }
}

impl From<winit::error::EventLoopError> for ApliteError {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoopCreationFailed(value)
    }
}

impl From<wgpu::CreateSurfaceError> for ApliteError {
    fn from(value: wgpu::CreateSurfaceError) -> Self {
        Self::SurfaceCreationFailed(value)
    }
}

impl From<wgpu::RequestDeviceError> for ApliteError {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        Self::DeviceRequestFailed(value)
    }
}

impl From<wgpu::RequestAdapterError> for ApliteError {
    fn from(value: wgpu::RequestAdapterError) -> Self {
        Self::AdapterRequestFailed(value)
    }
}

impl From<winit::error::OsError> for ApliteError {
    fn from(value: winit::error::OsError) -> Self {
        Self::WindowCreationFailed(value)
    }
}
