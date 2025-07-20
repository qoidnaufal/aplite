use std::sync::Arc;
use winit::window::Window;
use super::RendererError;

pub struct GpuDevice {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuDevice {
    pub async fn new(adapter: &wgpu::Adapter) -> Result<Self, RendererError> {
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                ..Default::default()
            },
        ).await?;
        Ok(Self { device, queue })
    }
}

pub struct GpuSurface {
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuSurface {
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let size = window.inner_size().to_logical(window.scale_factor());

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await?;

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

        Ok(Self { surface, adapter, config })
    }

    pub async fn create_device_queue(&self) -> Result<GpuDevice, RendererError> {
        GpuDevice::new(&self.adapter).await
    }
}

#[inline]
const fn backend() -> wgpu::Backends {
    #[cfg(all(unix, not(target_os = "macos")))]
    return wgpu::Backends::GL;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}
