use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::{PhysicalPosition, PhysicalSize, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_reactive::{Effect, Update, With};
use aplite_types::{Size, Rgba};
use aplite_renderer::{GpuDevice, GpuSurface, Renderer, RendererError};
use aplite_future::block_on;

use crate::prelude::ApliteResult;
use crate::context::Context;
use crate::error::ApliteError;
use crate::view::{IntoView, View, ViewId, VIEW_STORAGE};

pub(crate) const DEFAULT_SCREEN_SIZE: LogicalSize<u32> = LogicalSize::new(800, 600);

pub(crate) struct WindowHandle {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) root_id: ViewId,
}

impl WindowHandle {
    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.configure(device);
    }

    pub(crate) fn configure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }
}

pub struct Aplite {
    cx: Context,
    renderer: Option<Renderer>,
    window: HashMap<WindowId, WindowHandle>,
    pending_views: Vec<Box<dyn FnOnce(WindowId) -> Box<dyn IntoView>>>,
    window_attributes_fn: Option<fn(&mut WindowAttributes)>,

    #[cfg(feature = "render_stats")]
    stats: aplite_stats::Stats,
}

// user API
impl Aplite {
    pub fn new<IV: IntoView + 'static>(view_fn: impl FnOnce() -> IV + 'static) -> Self {
        let mut app = Self::new_empty();
        app.pending_views.push(Box::new(|_| Box::new(view_fn())));
        app
    }

    pub fn new_empty() -> Self {
        Self {
            renderer: None,
            cx: Context::new(),
            window: HashMap::with_capacity(4),
            window_attributes_fn: None,
            pending_views: Vec::with_capacity(4),

            #[cfg(feature = "render_stats")]
            stats: aplite_stats::Stats::new(),
        }
    }

    pub fn launch(mut self) -> ApliteResult {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
    }

    pub fn set_window_attributes(mut self, f: fn(&mut WindowAttributes)) -> Self {
        self.window_attributes_fn = Some(f);
        self
    }

    // pub fn with_background_color(self, color: Rgba<u8>) -> Self {
    //     let _ = color;
    //     self
    // }
}

// initialization
impl Aplite {
    fn initialize_window_and_renderer(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ApliteError> {
        let mut attributes = WindowAttributes::default()
            .with_inner_size(DEFAULT_SCREEN_SIZE)
            .with_title("Aplite Window");
        if let Some(window_fn) = self.window_attributes_fn.take() {
            window_fn(&mut attributes);
        }
        let window = event_loop.create_window(attributes)?;
        let window = Arc::new(window);
        let window_id = window.id();
        let size = window
            .inner_size()
            .to_logical(window.scale_factor());

        let root_id = VIEW_STORAGE.with(|s| {
            let root = s.create_entity();
            let root_view = View::window(Size::new(size.width, size.height));

            s.storage.borrow_mut().insert(root, root_view);

            if let Some(view_fn) = self.pending_views.pop() {
                let view = view_fn(window_id);
                view.widget_state().z_index.update(|z_index| *z_index += 1);
                s.append_child(&root, view);

                self.cx.layout_the_whole_window(&root);

                #[cfg(feature = "debug_tree")] eprintln!("{:?}", s.tree.borrow());
            }

            root
        });

        let gpu_surface = block_on(async {
            GpuSurface::new(Arc::clone(&window)).await
        })?;

        let window_handle = WindowHandle {
            window: Arc::clone(&window),
            surface: gpu_surface.surface,
            config: gpu_surface.config,
            root_id,
        };

        if let Some(renderer) = self.renderer.as_ref() {
            window_handle.configure(&renderer.device);
        } else {
            let gpu_device = block_on(async { GpuDevice::new(&gpu_surface.adapter).await})?;
            let renderer = Renderer::new(
                gpu_device.device,
                gpu_device.queue,
                Size::new(size.width as f32, size.height as f32),
                window.scale_factor()
            );
            window_handle.configure(&renderer.device);
            self.renderer = Some(renderer);
        }

        self.window.insert(window_id, window_handle);
        self.track_window(root_id, window);

        Ok(())
    }

    /// Track the [`Window`] with the associated root [`ViewId`] for rendering
    fn track_window(&mut self, view_id: ViewId, window: Arc<Window>) {
        let dirty = self.cx.dirty();

        Effect::new(move |_| {
            dirty.with(|root_id| {
                if root_id.is_some_and(|id| id == view_id) {
                    window.request_redraw();
                }
            })
        });
    }
}

// window event
impl Aplite {
    fn handle_resize(&mut self, size: PhysicalSize<u32>, window_id: WindowId) {
        if let Some(renderer) = self.renderer.as_mut()
        && let Some(window_handle) = self.window.get_mut(&window_id)
        && size.width > 0 && size.height > 0
        {
            let logical = size.to_logical::<u32>(renderer.scale_factor());
            // let root_id = self.root_view_id[&window_id];
            // VIEW_STORAGE.with(|s| {
            //     if let Some(window_state) = s.storage.borrow().get(&root_id) {
            //         window_state.widget_state().rect.update_untracked(|rect| {
            //             rect.set_size(Size::new(logical.width, logical.height));
            //         });
            //     }
            // });
            // crate::context::layout::LayoutContext::new(root_id).calculate();
            window_handle.resize(&renderer.device, logical.width, logical.height);
            renderer.resize(Size::new(logical.width as f32, logical.height as f32));
        }
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_scale_factor(scale_factor);
        }
    }

    fn handle_mouse_move(&mut self, window_id: &WindowId, pos: PhysicalPosition<f64>) {
        if let Some(renderer) = self.renderer.as_mut()
        && let Some(WindowHandle { root_id, .. }) = self.window.get(window_id) {
            let logical_pos = pos.to_logical::<f32>(renderer.scale_factor());
            self.cx.handle_mouse_move(root_id, (logical_pos.x, logical_pos.y));
        }
    }

    fn handle_click(&mut self, state: ElementState, button: MouseButton) {
        self.cx.handle_click(state, button);
    }

    fn handle_close_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.remove(window_id) {
            drop(window);
            event_loop.exit();
        }
    }

    // WARN: not sure if retained mode works like this
    fn handle_redraw_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window_handle) = self.window.get(window_id)
        && let Some(renderer) = self.renderer.as_mut()
        && let Ok(surface) = window_handle.surface.get_current_texture()
        {
            #[cfg(feature = "render_stats")] let start = std::time::Instant::now();

            renderer.begin();
            self.cx.prepare_data(window_handle.root_id, renderer);
            let format = window_handle.config.format;
            // TODO: this should be window.pre_present_notify(),
            // and the renderer.finish()
            if let Err(err) = renderer.render(
                Rgba::TRANSPARENT,
                Arc::clone(&window_handle.window),
                surface,
                format
            ) {
                let size = renderer.screen_res();
                match err {
                    RendererError::ShouldResize => renderer.resize(size),
                    RendererError::ShouldExit => event_loop.exit(),
                    _ => {}
                }
            }

            #[cfg(feature = "render_stats")] self.stats.inc(start.elapsed());
        }
    }
}

impl ApplicationHandler for Aplite {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.initialize_window_and_renderer(event_loop)
            .unwrap_or_else(|_| event_loop.exit())
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.handle_close_request(&window_id, event_loop),
            WindowEvent::RedrawRequested => self.handle_redraw_request(&window_id, event_loop),
            WindowEvent::Resized(s) => self.handle_resize(s, window_id),
            WindowEvent::MouseInput { state, button, .. } => self.handle_click(state, button),
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(&window_id, position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.set_scale_factor(scale_factor),
            _ => {}
        }
    }
}

// pub struct WindowAttributes {
//     title: &'static str,
//     inner_size: Size,
//     decorations: bool,
//     transparent: bool,
//     maximized: bool,
//     resizable: bool,
// }

// impl Default for WindowAttributes {
//     fn default() -> Self {
//         Self {
//             title: "GUI App",
//             inner_size: DEFAULT_SCREEN_SIZE,
//             decorations: true,
//             transparent: false,
//             maximized: false,
//             resizable: true,
//         }
//     }
// }

// #[cfg(target_os = "macos")]
// impl From<&WindowAttributes> for winit::window::WindowAttributes {
//     fn from(w: &WindowAttributes) -> Self {
//         use winit::platform::macos::WindowAttributesExtMacOS;

//         Self::default()
//             .with_inner_size(LogicalSize::new(w.inner_size.width as u32, w.inner_size.height as u32))
//             .with_title(w.title)
//             .with_transparent(w.transparent)
//             .with_maximized(w.maximized)
//             .with_resizable(w.resizable)
//             .with_titlebar_hidden(!w.decorations)
//     }
// }

// #[cfg(not(target_os = "macos"))]
// impl From<&WindowAttributes> for winit::window::WindowAttributes {
//     fn from(w: &WindowAttributes) -> Self {
//         Self::default()
//             .with_inner_size(LogicalSize::new(w.inner_size.width(), w.inner_size.height()))
//             .with_title(w.title)
//             .with_transparent(w.transparent)
//             .with_maximized(w.maximized)
//             .with_resizable(w.resizable)
//             .with_decorations(w.decorations)
//     }
// }
