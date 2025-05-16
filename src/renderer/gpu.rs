use std::sync::Arc;

use shared::Size;
use winit::window::Window;

use crate::error::ApliteError;

pub(crate) struct Gpu {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

fn backend() -> wgpu::Backends {
    #[cfg(all(unix, not(target_os = "macos")))]
    return wgpu::Backends::GL;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}

impl Gpu {
    pub(crate) fn new(window: Arc<Window>) -> Result<Self, ApliteError> {
        let scale_factor = window.scale_factor();
        let size = window.inner_size().to_logical(scale_factor);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }).await.map_err(ApliteError::AdapterRequestFailed)?;
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    ..Default::default()
                },
            ).await?;

            Ok::<(wgpu::Adapter, wgpu::Device, wgpu::Queue), ApliteError>((adapter, device, queue))
        })?;

        let surface_capabilites = surface.get_capabilities(&adapter);
        let format = surface_capabilites
            .formats
            .iter()
            .find(|f| matches!(f, wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb))
            .copied()
            .unwrap_or(surface_capabilites.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            // scale_factor,
        })
    }

    pub(crate) fn reconfigure_size(&mut self, size: Size<u32>) {
        self.config.width = size.width();
        self.config.height = size.height();
        self.configure();
    }

    #[inline(always)]
    pub(crate) fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    #[inline(always)]
    pub(crate) fn configure(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    /// this one uses [`winit::dpi::LogicalSize<u32>`]
    #[inline(always)]
    pub(crate) fn size(&self) -> Size<u32> {
        Size::new(self.config.width, self.config.height)
    }
}
