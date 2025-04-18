use std::sync::Arc;

use util::Size;
use winit::window::Window;

use crate::error::Error;

pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

fn backend() -> wgpu::Backends {
    #[cfg(all(unix, not(target_os = "macos")))]
    return wgpu::Backends::GL;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}

impl Gpu {
    pub fn request(window: Arc<Window>) -> Result<Self, Error> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }).await.map_err(Error::AdapterRequestFailed)?;
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    ..Default::default()
                },
            ).await?;

            Ok::<(wgpu::Adapter, wgpu::Device, wgpu::Queue), Error>((adapter, device, queue))
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
        })
    }

    pub fn configure(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn size(&self) -> Size<u32> {
        Size::new(self.config.width, self.config.height)
    }
}
