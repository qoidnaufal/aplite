use aplite_renderer::{RenderError, InitiationError};

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum ApliteError {
    EventLoopCreationFailed(winit::error::EventLoopError),
    WindowCreationFailed(winit::error::OsError),
    RenderError(RenderError),
    InitiationError(InitiationError),
}

impl std::fmt::Display for ApliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoopCreationFailed(err) => write!(f, "{err:?}"),
            Self::WindowCreationFailed(err) => write!(f, "{err:?}"),
            Self::RenderError(err) => write!(f, "{err:?}"),
            Self::InitiationError(err) => write!(f, "{err:?}"),
        }
    }
}

impl std::error::Error for ApliteError {}

impl From<winit::error::EventLoopError> for ApliteError {
    fn from(value: winit::error::EventLoopError) -> Self {
        Self::EventLoopCreationFailed(value)
    }
}

impl From<winit::error::OsError> for ApliteError {
    fn from(value: winit::error::OsError) -> Self {
        Self::WindowCreationFailed(value)
    }
}

impl From<RenderError> for ApliteError {
    fn from(value: RenderError) -> Self {
        Self::RenderError(value)
    }
}

impl From<InitiationError> for ApliteError {
    fn from(value: InitiationError) -> Self {
        Self::InitiationError(value)
    }
}
