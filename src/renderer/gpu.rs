use util::Size;
use winit::window::Window;

use crate::error::Error;

pub struct Gpu<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

#[cfg(all(not(target_os = "macos"), unix))]
fn backend() -> wgpu::Backends {
    wgpu::Backends::GL
}

#[cfg(target_os = "macos")]
fn backend() -> wgpu::Backends {
    wgpu::Backends::METAL
}

impl<'a> Gpu<'a> {
    pub fn request(window: &'a Window) -> Result<Self, Error> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }).await.ok_or(Error::NoAdapterFound)?;
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            }, None).await?;

            Ok::<(wgpu::Adapter, wgpu::Device, wgpu::Queue), Error>((adapter, device, queue))
        })?;

        let surface_capabilites = surface.get_capabilities(&adapter);
        let format = surface_capabilites
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilites.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilites.present_modes[0],
            alpha_mode: surface_capabilites.alpha_modes[0],
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
