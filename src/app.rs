use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::{PhysicalPosition, PhysicalSize, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_reactive::*;
use aplite_types::Size;
use aplite_renderer::Renderer;
use aplite_future::block_on;

use crate::prelude::ApliteResult;
use crate::context::Context;
use crate::error::ApliteError;
use crate::view::{IntoView, View, ViewId, VIEW_STORAGE};

pub(crate) const DEFAULT_SCREEN_SIZE: LogicalSize<u32> = LogicalSize::new(800, 600);

pub(crate) struct WindowHandle {
    pub(crate) window: Arc<Window>,
    pub(crate) root_id: ViewId,
}

pub struct Aplite {
    cx: Context,
    renderer: Option<Renderer>,
    window: HashMap<WindowId, WindowHandle>,
    pending_views: Option<Box<dyn FnOnce(WindowId) -> Box<dyn IntoView>>>,
    window_attributes_fn: Option<fn(&mut WindowAttributes)>,

    #[cfg(feature = "render_stats")]
    stats: aplite_stats::Stats,
}

// user API
impl Aplite {
    pub fn new<IV: IntoView + 'static>(view_fn: impl FnOnce() -> IV + 'static) -> Self {
        let mut app = Self::new_empty();
        app.pending_views = Some(Box::new(|_| Box::new(view_fn())));
        app
    }

    pub fn new_empty() -> Self {
        Self {
            renderer: None,
            cx: Context::new(),
            window: HashMap::with_capacity(4),
            window_attributes_fn: None,
            pending_views: None,

            #[cfg(feature = "render_stats")]
            stats: aplite_stats::Stats::new(),
        }
    }

    // FIXME: figure out how to integrate async runtime here
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
            let root_view = View::window(Size::new(size.width, size.height));
            let root_id = root_view.node.id();

            s.storage.borrow_mut().insert(root_id, root_view);

            if let Some(view_fn) = self.pending_views.take() {
                let view = view_fn(window_id);
                s.append_child(&root_id, view);

                self.cx.layout_the_whole_window(&root_id);

                #[cfg(feature = "debug_tree")] eprintln!("{:?}", s.tree.borrow());
            }

            root_id
        });

        let window_handle = WindowHandle {
            window: Arc::clone(&window),
            root_id,
        };

        let renderer = block_on(async { Renderer::new(Arc::clone(&window)).await })?;

        self.renderer = Some(renderer);
        self.window.insert(window_id, window_handle);
        self.track_window(window);

        Ok(())
    }

    /// Track the [`Window`] with the associated root [`ViewId`] for rendering
    fn track_window(&mut self, window: Arc<Window>) {
        let dirty = Context::dirty();

        Effect::new(move |_| if dirty.get() { window.request_redraw() });
    }
}

// window event
impl Aplite {
    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(renderer) = self.renderer.as_mut()
        && size.width > 0 && size.height > 0
        {
            renderer.resize(size);
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
        {
            #[cfg(feature = "render_stats")] let start = std::time::Instant::now();

            match renderer.begin() {
                Ok(()) => {
                    self.cx.prepare_data(window_handle.root_id, renderer.new_scene());
                    renderer.encode();
                    window_handle.window.pre_present_notify();
                    renderer.finish();
                },
                Err(err) => match err {
                    aplite_renderer::RenderError::ShouldResize => renderer
                        .resize(window_handle.window.inner_size()),
                    aplite_renderer::RenderError::ShouldExit => event_loop.exit(),
                    _ => {}
                },
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
            WindowEvent::Resized(size) => self.handle_resize(size),
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
