use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::{PhysicalPosition, PhysicalSize, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_reactive::*;
use aplite_types::{Size, Rect};
use aplite_renderer::Renderer;
use aplite_future::{block_on, Executor};

use crate::prelude::ApliteResult;
use crate::context::Context;
use crate::context::layout::LayoutCx;
use crate::error::ApliteError;
use crate::view::{IntoView, ViewId, Render};
use crate::widget::WindowWidget;

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

    pub fn launch(mut self) -> ApliteResult {
        Executor::init();
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
            .to_logical::<f32>(window.scale_factor());
        let bound = Rect::from_size(Size::new(size.width, size.height));
        let window_widget = WindowWidget::new(bound);

        let root_id = {
            if let Some(view_fn) = self.pending_views.take() {
                let view = view_fn(window_id);

                view.calculate_size(None);
                let mut cx = LayoutCx::new(&window_widget);
                view.calculate_layout(&mut cx);

                #[cfg(feature = "debug_tree")] eprintln!("{:?}", view);

                self.cx.insert_view(view.into_view())
            } else {
                self.cx.insert_view(window_widget.into_view())
            }
        };

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
        let dirty = self.cx.dirty();

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
            && let Some(WindowHandle { root_id, .. }) = self.window.get(window_id)
        {
            let logical_pos = pos.to_logical::<f32>(renderer.scale_factor());
            self.cx.handle_mouse_move(root_id, (logical_pos.x, logical_pos.y));
        }
    }

    fn handle_click(&mut self, window_id: &WindowId, state: ElementState, button: MouseButton) {
        if let Some(WindowHandle { root_id, .. }) = self.window.get(window_id) {
            self.cx.handle_click(root_id, state, button);
        }
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
                    self.cx.render(window_handle.root_id, renderer);
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
            .unwrap_or_else(|_| event_loop.exit());
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
            WindowEvent::MouseInput { state, button, .. } => self.handle_click(&window_id, state, button),
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(&window_id, position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.set_scale_factor(scale_factor),
            _ => {}
        }
    }
}
