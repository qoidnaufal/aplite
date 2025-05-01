use std::collections::HashMap;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use util::{Size, Vector2};

use crate::color::Pixel;
use crate::cursor::Cursor;
use crate::prelude::AppResult;
use crate::renderer::{Gfx, Gpu, IntoRenderSource, Renderer};
use crate::context::Context;
use crate::error::GuiError;
use crate::view::{IntoView, Render};

struct Stats {
    counter: u32,
    time: std::time::Duration,
}

impl Stats {
    fn new() -> Self {
        Self {
            counter: 0,
            time: std::time::Duration::from_nanos(0),
        }
    }

    fn inc(&mut self, d: std::time::Duration) {
        self.time += d;
        self.counter += 1;
    }
}

impl Drop for Stats {
    fn drop(&mut self) {
        let avg = self.time / self.counter;
        println!("average render time: {avg:?}");
    }
}

pub struct App<F> {
    renderer: Option<Renderer>,
    cx: Context,
    cursor: Cursor,
    window: HashMap<WindowId, Arc<Window>>,
    window_fn: Option<fn(&Window)>,
    stats: Stats,
    view_fn: Option<F>,
}

impl<F, IV> App<F>
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView + 'static,
{
    pub fn new(view_fn: F) -> Self {
        let mut app = Self::new_empty();
        app.view_fn = Some(view_fn);
        app
    }

    pub fn new_empty() -> Self {
        Self {
            renderer: None,
            cx: Context::default(),
            cursor: Cursor::new(),
            window: HashMap::with_capacity(4),
            window_fn: None,
            stats: Stats::new(),
            view_fn: None,
        }
    }

    pub fn launch(mut self) -> AppResult {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
    }

    pub fn set_window_properties(mut self, f: fn(&Window)) -> Self {
        self.window_fn = Some(f);
        self
    }

    fn initialize_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) -> Arc<Window> {
        let window = event_loop.create_window(Default::default()).unwrap();
        window.set_title("GUI App");
        if let Some(window_fn) = self.window_fn.take() {
            window_fn(&window);
        }
        self.cx.initialize_root(&window);
        Arc::new(window)
    }

    fn initialize_renderer(&mut self, window: Arc<Window>) {
        let gpu = Gpu::request(window.clone()).unwrap();
        let mut gfx = Gfx::new(&gpu.device);
        gfx.register(&gpu, None::<Pixel<u8>>, self.cx.get_window_properties());
        if let Some(view_fn) = self.view_fn.take() {
            view_fn().into_view().render(&mut self.cx);
            self.cx.register(&gpu, &mut gfx);
        }
        self.renderer = Some(Renderer::new(gpu, gfx));
    }

    fn request_redraw(&self, window_id: winit::window::WindowId) {
        if let Some(window) = self.window.get(&window_id) {
            window.request_redraw();
        }
    }

    fn resize(&mut self, size: impl Into<Size<u32>>) {
        if let Some(renderer) = self.renderer.as_mut() {
            let size: Size<u32> = size.into();
            self.cx.update_window_properties(|prop| prop.set_size(size));
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

    fn render(&mut self) -> Result<(), GuiError> {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.render()
        } else {
            Err(GuiError::UnitializedRenderer)
        }
    }
}

impl<F, IV> ApplicationHandler for App<F>
where
    F: FnOnce() -> IV + 'static,
    IV: IntoView + 'static,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = self.initialize_window(event_loop);
        self.initialize_renderer(Arc::clone(&window));
        self.window.insert(window.id(), window);

        // self.cx.print_nodes();
        // eprintln!("{}", self.cx.trees.len());
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

                let start = std::time::Instant::now();
                match self.render() {
                    Ok(_) => {},
                    Err(GuiError::SurfaceRendering(surface_err)) => {
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
                let elapsed = start.elapsed();
                self.stats.inc(elapsed);
            }
            WindowEvent::Resized(new_size) => {
                self.resize((new_size.width, new_size.height));
            }
            WindowEvent::MouseInput { state: action, button, .. } => {
                let gfx = &mut self.renderer.as_mut().unwrap().gfx;
                self.cursor.set_click_state(action.into(), button.into());
                self.cx.handle_click(&mut self.cursor, gfx);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let renderer = self.renderer.as_mut().unwrap();
                self.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                if !renderer.gfx.is_empty() {
                    self.cx.detect_hover(&mut self.cursor);
                    self.cx.handle_hover(&mut self.cursor, &mut renderer.gfx);
                }
            }
            _ => {}
        }
        self.detect_update(window_id);
    }
}

