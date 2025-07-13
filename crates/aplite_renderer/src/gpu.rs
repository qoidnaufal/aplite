use std::sync::Arc;

use aplite_types::Size;
use aplite_future::block_on;
use winit::window::Window;

use super::RendererError;

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
    pub(crate) fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let scale_factor = window.scale_factor();
        let size = window.inner_size().to_logical(scale_factor);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: backend(),
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;
        let (adapter, device, queue) = block_on(async {
            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }).await?;
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    ..Default::default()
                },
            ).await?;

            Ok::<(wgpu::Adapter, wgpu::Device, wgpu::Queue), RendererError>((adapter, device, queue))
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

        surface.configure(&device, &config);
        Ok(Self { surface, device, queue, config })
    }

    pub(crate) fn reconfigure_size(&mut self, size: Size<u32>) {
        self.config.width = size.width();
        self.config.height = size.height();
        self.surface.configure(&self.device, &self.config);
    }

    #[inline(always)]
    pub(crate) fn get_surface_texture(&self) -> Result<wgpu::SurfaceTexture, RendererError> {
        let surface = self.surface.get_current_texture()?;
        Ok(surface)
    }

    /// this one uses [`winit::dpi::LogicalSize<u32>`]
    #[inline(always)]
    pub(crate) fn size(&self) -> Size<u32> {
        Size::new(self.config.width, self.config.height)
    }

    pub(crate) fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        let label = Some("render encoder");
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label })
    }

    pub(crate) fn submit_encoder(&self, encoder: wgpu::CommandEncoder) -> wgpu::SubmissionIndex {
        self.queue.submit([encoder.finish()])
    }

    pub(crate) fn poll_wait(&self, index: wgpu::SubmissionIndex) -> Result<(), RendererError> {
        self.device.poll(wgpu::PollType::WaitForSubmissionIndex(index))?;
        Ok(())
    }
}
