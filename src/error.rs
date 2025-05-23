use aplite_renderer::RendererError;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum ApliteError {
    EventLoopCreationFailed(winit::error::EventLoopError),
    WindowCreationFailed(winit::error::OsError),
    RendererError(RendererError),
}

impl std::fmt::Display for ApliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoopCreationFailed(err) => write!(f, "{err:?}"),
            Self::WindowCreationFailed(err) => write!(f, "{err:?}"),
            Self::RendererError(err) => write!(f, "{err:?}"),
        }
    }
}

impl std::error::Error for ApliteError {}

impl From<winit::error::EventLoopError> for ApliteError {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoopCreationFailed(value)
    }
}

impl From<RendererError> for ApliteError {
    fn from(value: RendererError) -> Self {
        Self::RendererError(value)
    }
}

impl From<winit::error::OsError> for ApliteError {
    fn from(value: winit::error::OsError) -> Self {
        Self::WindowCreationFailed(value)
    }
}
