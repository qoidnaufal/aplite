use std::collections::HashMap;
use std::sync::{Arc, Weak};

use winit::dpi::{PhysicalPosition, PhysicalSize, LogicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_reactive::*;
use aplite_renderer::Renderer;
use aplite_future::block_on;

use crate::prelude::ApliteResult;
use crate::context::Context;
// use crate::layout::{LayoutCx, Layout};
use crate::error::ApliteError;
use crate::view::IntoView;

pub(crate) const DEFAULT_SCREEN_SIZE: LogicalSize<u32> = LogicalSize::new(800, 600);

pub(crate) struct WindowHandle {
    pub(crate) window: Arc<Window>,
}

pub struct Aplite {
    cx: Context,
    renderer: Option<Renderer>,
    window_handle: HashMap<WindowId, WindowHandle>,
    window_attributes_fn: Option<fn(&mut WindowAttributes)>,

    #[cfg(feature = "render_stats")]
    stats: aplite_stats::Stats,
}

// user API
impl Aplite {
    pub fn new<IV: IntoView + 'static>(view_fn: impl FnOnce() -> IV + 'static) -> Self {
        // Executor::init();
        Self {
            renderer: None,
            cx: Context::new(view_fn().into_view()),
            window_handle: HashMap::with_capacity(4),
            window_attributes_fn: None,

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

        // let size = window
        //     .inner_size()
        //     .to_logical::<f32>(window.scale_factor());
        // let bound = Rect::from_size(Size::new(size.width, size.height));
        // let window_widget = WindowWidget::new(&mut self.cx, bound);
        // let mut cx = LayoutCx::new(&window_widget);

        // self.cx.view.widget.calculate_size(None);
        // self.cx.view.widget.calculate_layout(&mut cx);

        self.cx.calculate_layout();
        #[cfg(feature = "debug_tree")] eprintln!("{:#?}", view);

        let renderer = block_on(async { Renderer::new(Arc::downgrade(&window)).await })?;

        self.renderer = Some(renderer);
        self.track_window(Arc::downgrade(&window));

        let window_handle = WindowHandle { window };
        self.window_handle.insert(window_id, window_handle);

        Ok(())
    }

    /// Track the [`Window`] with the associated root [`ViewId`] for rendering
    fn track_window(&mut self, window: Weak<Window>) {
        let dirty = self.cx.dirty;

        Effect::new(move |_| {
            if dirty.get() && let Some(window) = window.upgrade() {
                window.request_redraw();
            }
        });
    }

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

    fn handle_mouse_move(&mut self, _window_id: &WindowId, pos: PhysicalPosition<f64>) {
        if let Some(renderer) = self.renderer.as_mut() {
            let logical_pos = pos.to_logical::<f32>(renderer.scale_factor());
            self.cx.handle_mouse_move((logical_pos.x, logical_pos.y));
        }
    }

    fn handle_click(&mut self, state: ElementState, button: MouseButton) {
        self.cx.handle_click(state, button);
    }

    fn handle_close_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window_handle.remove(window_id) {
            drop(window);
            event_loop.exit();
        }
    }

    // WARN: not sure if retained mode works like this
    fn handle_redraw_request(&mut self, window_id: &WindowId, _: &ActiveEventLoop) {
        if let Some(window_handle) = self.window_handle.get(window_id)
            && let Some(renderer) = self.renderer.as_mut()
        {
            #[cfg(feature = "render_stats")] let start = std::time::Instant::now();

            renderer.begin();
            self.cx.render(renderer);
            renderer.finish(window_handle.window.as_ref());

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
            WindowEvent::MouseInput { state, button, .. } => self.handle_click(state, button),
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(&window_id, position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.set_scale_factor(scale_factor),
            _ => {}
        }

        if let Some(_handle) = self.window_handle.get(&window_id) {
            self.cx.process_pending_update();
        }
    }
}
