use std::collections::HashMap;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use shared::{Size, Vector2, Rgba};

use crate::prelude::ApliteResult;
use crate::renderer::util::Render;
use crate::renderer::Renderer;
use crate::context::Context;
use crate::error::ApliteError;

#[cfg(feature = "stats")]
mod stats {
    pub(crate) struct Stats {
        counter: u32,
        render_time: std::time::Duration,
        startup_time: std::time::Duration,
        longest: std::time::Duration,
        shortest: std::time::Duration,
    }

    impl Stats {
        pub(crate) fn new() -> Self {
            Self {
                counter: 0,
                render_time: std::time::Duration::from_nanos(0),
                startup_time: std::time::Duration::from_nanos(0),
                longest: std::time::Duration::from_nanos(0),
                shortest: std::time::Duration::from_nanos(0),
            }
        }

        pub(crate) fn inc(&mut self, d: std::time::Duration) {
            if self.counter == 0 {
                self.startup_time += d;
            } else if self.counter == 1 {
                self.longest = self.longest.max(d);
                self.shortest = d;
                self.render_time += d;
            } else {
                self.longest = self.longest.max(d);
                self.shortest = self.shortest.min(d);
                self.render_time += d;
            }
            self.counter += 1;
        }
    }

    impl Drop for Stats {
        fn drop(&mut self) {
            if self.counter == 1 {
                let startup = self.startup_time;
                eprintln!("startup time: {startup:?}");
            } else {
                let counter = self.counter - 1;
                let startup = self.startup_time;
                let render = self.render_time / counter;
                eprintln!("startup:             {startup:?}");
                eprintln!("average:             {render:?}");
                eprintln!("longest:             {:?}", self.longest);
                eprintln!("shortest:            {:?}", self.shortest);
                eprintln!("render amount:       {counter}");
            }
        }
    }
}

pub(crate) const DEFAULT_SCREEN_SIZE: Size<u32> = Size::new(1600, 1200);

pub struct WindowAttributes {
    title: String,
    inner_size: Size<u32>,
    decorations: bool,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: "GUI App".into(),
            inner_size: DEFAULT_SCREEN_SIZE,
            decorations: true,
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
}

impl From<WindowAttributes> for winit::window::WindowAttributes {
    fn from(w: WindowAttributes) -> Self {
        Self::default()
            .with_inner_size::<winit::dpi::PhysicalSize<u32>>(w.inner_size.into())
            .with_title(w.title)
            .with_decorations(w.decorations)
    }
}

pub struct Aplite<F: FnOnce(&mut Context)> {
    renderer: Option<Renderer>,
    cx: Context,
    window: HashMap<WindowId, Arc<Window>>,
    window_fn: Option<fn(&mut WindowAttributes)>,
    #[cfg(feature = "stats")]
    stats: stats::Stats,
    view_fn: Option<F>,
}

impl<F: FnOnce(&mut Context)> Aplite<F> {
    pub fn new(view_fn: F) -> Self {
        let mut app = Self::new_empty();
        app.view_fn = Some(view_fn);
        app
    }

    pub fn new_empty() -> Self {
        let mut cx = Context::default();
        cx.initialize_root(DEFAULT_SCREEN_SIZE);
        Self {
            renderer: None,
            cx,
            window: HashMap::with_capacity(4),
            window_fn: None,
            #[cfg(feature = "stats")]
            stats: stats::Stats::new(),
            view_fn: None,
        }
    }

    pub fn launch(mut self) -> ApliteResult {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
    }

    pub fn set_window_attributes(mut self, f: fn(&mut WindowAttributes)) -> Self {
        self.window_fn = Some(f);
        self
    }

    pub fn set_background_color(mut self, color: Rgba<u8>) -> Self {
        self.cx.update_window_properties(|prop| prop.set_fill_color(color));
        self
    }

    fn initialize_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Arc<Window>, ApliteError> {
        let mut attributes = WindowAttributes::default();
        if let Some(window_fn) = self.window_fn.take() {
            window_fn(&mut attributes);
        }
        let window = event_loop.create_window(attributes.into())?;
        let inner_size = window.inner_size();
        let size: Size<u32> = (inner_size.width, inner_size.height).into();
        self.cx.update_window_properties(|prop| {
            prop.set_size(size);
            prop.set_position((size / 2).into());
        });
        Ok(Arc::new(window))
    }

    fn initialize_renderer(&mut self, window: Arc<Window>) -> Result<(), ApliteError> {
        let mut renderer = Renderer::new(Arc::clone(&window))?;
        if let Some(view_fn) = self.view_fn.take() {
            view_fn(&mut self.cx);
            self.cx.render(&mut renderer);
            self.cx.layout();
            self.cx.debug_tree();
        }
        self.renderer = Some(renderer);
        Ok(())
    }

    fn add_window(&mut self, window: Arc<Window>) {
        self.window.insert(window.id(), window);
    }

    fn request_redraw(&self, window_id: winit::window::WindowId) {
        if let Some(window) = self.window.get(&window_id) {
            window.request_redraw();
        }
    }

    fn resize(&mut self, size: impl Into<Size<u32>>) {
        if let Some(renderer) = self.renderer.as_mut() {
            let size: Size<u32> = size.into();
            self.cx.update_window_properties(|wp| {
                wp.set_size(size);
                wp.set_position((size / 2).into());
            });
            // if !renderer.model.is_unitialized() { self.cx.recursive_layout(&NodeId::root()) }
            renderer.resize(size);
        }
    }

    fn update(&mut self) {
        if let Some(renderer) = self.renderer.as_mut() {
            self.cx.submit_update(renderer);
        }
    }

    fn detect_update(&self, window_id: winit::window::WindowId) {
        if self.cx.has_changed() {
            self.request_redraw(window_id);
        }
    }

    fn render(&mut self) -> Result<(), ApliteError> {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.render(self.cx.get_window_properties().fill_color())
        } else {
            Err(ApliteError::UnitializedRenderer)
        }
    }
}

impl<F: FnOnce(&mut Context)> ApplicationHandler for Aplite<F> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Ok(window) = self.initialize_window(event_loop) {
            match self.initialize_renderer(Arc::clone(&window)) {
                Ok(_) => self.add_window(window),
                Err(_) => event_loop.exit(),
            }
        } else {
            event_loop.exit();
        }

        // eprintln!("{:?}", self.cx.tree);
        // eprintln!("current: {:?}", self.cx.current_entity());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let size = self.renderer.as_ref().unwrap().gpu.size();
        match event {
            WindowEvent::CloseRequested => {
                eprintln!();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.update();

                #[cfg(feature = "stats")]
                let start = std::time::Instant::now();

                match self.render() {
                    Ok(_) => {},
                    Err(ApliteError::SurfaceRendering(surface_err)) => {
                        match surface_err {
                            wgpu::SurfaceError::Outdated
                            | wgpu::SurfaceError::Lost => {
                                eprintln!("surface lost / outdated");
                                self.resize(size);
                            },
                            wgpu::SurfaceError::OutOfMemory
                            | wgpu::SurfaceError::Other => {
                                eprintln!("Out of Memory / other error");
                                event_loop.exit();
                            },
                            wgpu::SurfaceError::Timeout => {
                                eprintln!("Surface Timeout")
                            },
                        }
                    }
                    Err(_) => event_loop.exit(),
                }
                #[cfg(feature = "stats")]
                {
                    let elapsed = start.elapsed();
                    self.stats.inc(elapsed);
                }
            }
            WindowEvent::Resized(new_size) => {
                self.resize(new_size);
            }
            WindowEvent::MouseInput { state: action, button, .. } => {
                self.cx.cursor.set_click_state(action.into(), button.into());
                self.cx.handle_click();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let renderer = self.renderer.as_mut().unwrap();
                self.cx.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                if !renderer.gfx.is_empty() {
                    self.cx.detect_hovered_ancestor();
                    if self.cx.cursor.ancestor.is_some() {
                        self.cx.detect_hovered_child();
                        self.cx.handle_hover();
                    }
                }
            }
            _ => {}
        }
        self.detect_update(window_id);
    }
}

