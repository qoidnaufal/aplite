#[derive(Debug)]
pub enum ApliteError {
    UnitializedRenderer,
    EventLoopCreation(winit::error::EventLoopError),
    SurfaceCreation(wgpu::CreateSurfaceError),
    SurfaceRendering(wgpu::SurfaceError),
    DeviceRequest(wgpu::RequestDeviceError),
    AdapterRequestFailed(wgpu::RequestAdapterError),
    WindowCreationFailed(winit::error::OsError),
}

impl std::fmt::Display for ApliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_kind = match self {
            Self::UnitializedRenderer => "uninitialized renderer".to_string(),
            Self::EventLoopCreation(err) => err.to_string(),
            Self::SurfaceCreation(err) => err.to_string(),
            Self::SurfaceRendering(err) => err.to_string(),
            Self::DeviceRequest(err) => err.to_string(),
            Self::AdapterRequestFailed(err) => err.to_string(),
            Self::WindowCreationFailed(err) => err.to_string(),
        };

        write!(f, "{}", err_kind)
    }
}

impl std::error::Error for ApliteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnitializedRenderer => None,
            Self::EventLoopCreation(err) => err.source(),
            Self::SurfaceCreation(err) => err.source(),
            Self::SurfaceRendering(err) => err.source(),
            Self::DeviceRequest(err) => err.source(),
            Self::AdapterRequestFailed(err) => err.source(),
            Self::WindowCreationFailed(err) => err.source(),
        }
    }
}

impl From<winit::error::EventLoopError> for ApliteError {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoopCreation(value)
    }
}

impl From<wgpu::CreateSurfaceError> for ApliteError {
    fn from(value: wgpu::CreateSurfaceError) -> Self {
        Self::SurfaceCreation(value)
    }
}

impl From<wgpu::RequestDeviceError> for ApliteError {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        Self::DeviceRequest(value)
    }
}

impl From<wgpu::SurfaceError> for ApliteError {
    fn from(value: wgpu::SurfaceError) -> Self {
        Self::SurfaceRendering(value)
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
