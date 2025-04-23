use std::collections::HashMap;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use util::{Size, Vector2};

use crate::cursor::Cursor;
use crate::prelude::AppResult;
use crate::renderer::Renderer;
use crate::context::Context;
use crate::error::GuiError;
use crate::view::IntoView;

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
    window_properties: Option<fn(&Window)>,
    stats: Stats,
    view_fn: Option<F>,
}

impl<F, IV> App<F>
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static,
{
    pub fn new(view_fn: F) -> Self {
        Self {
            renderer: None,
            window: HashMap::with_capacity(4),
            window_properties: None,
            cx: Context::new(),
            cursor: Cursor::new(),
            stats: Stats::new(),
            view_fn: Some(view_fn),
        }
    }

    pub fn launch(mut self) -> AppResult {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
    }

    pub fn set_window_properties(mut self, f: fn(&Window)) -> Self {
        self.window_properties = Some(f);
        self
    }

    fn request_redraw(&self, window_id: winit::window::WindowId) {
        if let Some(window) = self.window.get(&window_id) {
            window.request_redraw();
        }
    }

    fn resize(&mut self, size: impl Into<Size<u32>>) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(size.into());
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

fn create_window(event_loop: &winit::event_loop::ActiveEventLoop) -> Arc<Window> {
    let window = event_loop.create_window(Default::default()).unwrap();
    Arc::new(window)
}

impl<F, IV> ApplicationHandler for App<F>
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(view_fn) = self.view_fn.take() {
            let window = create_window(event_loop);
            if let Some(window_fn) = self.window_properties.take() {
                window_fn(&window);
            } else {
                window.set_title("GUI App");
            }
            self.renderer = Some(Renderer::new(window.clone(), &mut self.cx, view_fn));
            self.window.insert(window.id(), window);
        }

        // eprintln!("{:#?}", self.cx.nodes);
        // eprintln!("{}", self.cx.nodes.len());
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
                self.cx.detect_hover(&mut self.cursor);
                self.cx.handle_hover(&mut self.cursor, &mut renderer.gfx);
            }
            _ => {}
        }
        self.detect_update(window_id);
    }
}

