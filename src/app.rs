use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_types::{Size, Rgba};
use aplite_renderer::{Render, Renderer, RendererError};

use crate::prelude::ApliteResult;
use crate::context::Context;
use crate::error::ApliteError;

#[derive(Debug)]
enum WinitSize {
    Logical(Size<u32>),
    Physical(PhysicalSize<u32>),
}

pub(crate) const DEFAULT_SCREEN_SIZE: Size<u32> = Size::new(800, 600);

pub struct WindowAttributes {
    title: String,
    inner_size: Size<u32>,
    decorations: bool,
    transparent: bool,
    maximized: bool,
    fullscreen: Option<winit::window::Fullscreen>,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: "GUI App".into(),
            inner_size: DEFAULT_SCREEN_SIZE,
            decorations: true,
            transparent: false,
            maximized: false,
            fullscreen: None,
        }
    }
}

impl WindowAttributes {
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_inner_size(&mut self, size: impl Into<Size<u32>>) {
        self.inner_size = size.into();
    }

    pub fn set_decorations_enabled(&mut self, val: bool) {
        self.decorations = val;
    }

    pub fn set_transparent(&mut self, val: bool) {
        self.transparent = val;
    }

    pub fn set_maximized(&mut self, val: bool) {
        self.maximized = val;
    }

    pub fn set_fullscreen_mode(&mut self, val: Option<winit::window::Fullscreen>) {
        self.fullscreen = val;
    }
}

impl From<&WindowAttributes> for winit::window::WindowAttributes {
    fn from(w: &WindowAttributes) -> Self {
        Self::default()
            .with_inner_size::<winit::dpi::LogicalSize<u32>>(w.inner_size.into())
            .with_title(&w.title)
            .with_decorations(w.decorations)
            .with_transparent(w.transparent)
            .with_maximized(w.maximized)
            .with_fullscreen(w.fullscreen.clone())
    }
}

pub struct Aplite {
    renderer: Option<Renderer>,
    cx: Context,
    window: HashMap<WindowId, Arc<Window>>,
    window_attributes: WindowAttributes,

    #[cfg(feature = "render_stats")]
    stats: aplite_stats::Stats,
}

// user API
impl Aplite {
    pub fn new<F: FnOnce(&mut Context)>(view_fn: F) -> Self {
        let mut app = Self::new_empty();
        view_fn(&mut app.cx);
        app.cx.layout();
        app
    }

    pub fn new_empty() -> Self {
        Self {
            renderer: None,
            cx: Context::new(DEFAULT_SCREEN_SIZE),
            window: HashMap::with_capacity(4),
            window_attributes: WindowAttributes::default(),

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
        f(&mut self.window_attributes);
        self
    }

    pub fn set_background_color(mut self, color: Rgba<u8>) -> Self {
        self.cx.update_window_properties(|prop| prop.set_fill_color(color));
        self
    }
}

// initialization
impl Aplite {
    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> Result<Arc<Window>, ApliteError> {
        let attributes = &self.window_attributes;
        let window = event_loop.create_window(attributes.into())?;
        let size: Size<u32> = window
            .inner_size()
            .to_logical(window.scale_factor())
            .into();

        self.cx.update_window_properties(|prop| {
            prop.set_size(size);
            prop.set_position((size / 2).into());
        });

        Ok(Arc::new(window))
    }

    fn initialize_renderer(&mut self, window: Arc<Window>) -> Result<(), ApliteError> {
        let mut renderer = Renderer::new(Arc::clone(&window))?;
        self.cx.render(&mut renderer);

        #[cfg(feature = "debug_tree")] self.cx.debug_tree();

        self.renderer = Some(renderer);
        Ok(())
    }

    fn add_window(&mut self, window: Arc<Window>) {
        self.window.insert(window.id(), window);
    }
}

// window event
impl Aplite {
    fn handle_resize(&mut self, winit_size: WinitSize) {
        if let Some(renderer) = self.renderer.as_mut() {
            let size = match winit_size {
                WinitSize::Logical(size) => size,
                WinitSize::Physical(size) => {
                    let logical = size.to_logical::<u32>(renderer.scale_factor());
                    (logical.width, logical.height).into()
                },
            };
            self.cx.update_window_properties(|wp| {
                wp.set_size(size);
                wp.set_position((size / 2).into());
            });
            renderer.resize(size);
        }
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_scale_factor(scale_factor);
        }
    }

    fn request_redraw(&self, window_id: &WindowId) {
        if let Some(window) = self.window.get(window_id) {
            window.request_redraw();
        }
    }

    fn detect_update(&mut self, window_id: &WindowId) {
        if self.cx.has_changed() {
            self.request_redraw(window_id);
        }
    }

    fn handle_redraw_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.get(window_id).cloned() {
            self.submit_update();

            #[cfg(feature = "render_stats")] let start = std::time::Instant::now();

            self.render(event_loop, || window.pre_present_notify());

            #[cfg(feature = "render_stats")] self.stats.inc(start.elapsed())
        }
    }

    fn submit_update(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            self.cx.submit_update(renderer);
        }
    }

    fn render<P: FnOnce()>(&mut self, event_loop: &ActiveEventLoop, pre_present_notify: P) {
        if self.renderer.is_none() { event_loop.exit() }
        let renderer = self.renderer.as_mut().unwrap();
        let size = renderer.surface_size();
        let color = self.cx.get_window_properties().fill_color();
        if let Err(err) = renderer.render(color, pre_present_notify) {
            match err {
                RendererError::ShouldResize => self.handle_resize(WinitSize::Logical(size)),
                RendererError::ShouldExit => event_loop.exit(),
                _ => {}
            }
        }
    }

    fn handle_close_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.remove(window_id) {
            drop(window);
            event_loop.exit();
        }
    }

    fn handle_click(&mut self, state: ElementState, button: MouseButton) {
        self.cx.handle_click(state, button);
    }

    fn handle_mouse_move(&mut self, pos: PhysicalPosition<f64>) {
        if self.renderer.is_none() { return }
        let renderer = self.renderer.as_mut().unwrap();
        let logical_pos = pos.to_logical::<f32>(renderer.scale_factor());
        self.cx.handle_mouse_move((logical_pos.x, logical_pos.y));
    }
}

impl ApplicationHandler for Aplite {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(window) = self.initialize_window(event_loop) {
            match self.initialize_renderer(Arc::clone(&window)) {
                Ok(_) => self.add_window(window),
                Err(_) => event_loop.exit(),
            }
        } else {
            event_loop.exit();
        }
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
            WindowEvent::Resized(s) => self.handle_resize(WinitSize::Physical(s)),
            WindowEvent::MouseInput { state, button, .. } => self.handle_click(state, button),
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.set_scale_factor(scale_factor),
            _ => {}
        }
        self.detect_update(&window_id);
    }
}

