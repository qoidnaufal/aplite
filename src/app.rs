use std::collections::HashMap;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use util::{Size, Vector2};

use crate::cursor::Cursor;
use crate::renderer::Renderer;
use crate::tree::WidgetTree;
use crate::error::Error;
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

pub(crate) struct App<F> {
    renderer: Option<Renderer>,
    tree: WidgetTree,
    cursor: Cursor,
    window: HashMap<WindowId, Arc<Window>>,
    stats: Stats,
    view_fn: Option<F>,
}

impl<F, IV> App<F>
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static,
{
    pub(crate) fn new(view_fn: F) -> Self {
        Self {
            renderer: None,
            window: HashMap::with_capacity(4),
            tree: WidgetTree::new(),
            cursor: Cursor::new(),
            stats: Stats::new(),
            view_fn: Some(view_fn),
        }
    }

    pub(crate) fn run(mut self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
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
            self.tree.submit_update(renderer);
        }
    }

    fn detect_update(&self, window_id: winit::window::WindowId) {
        if self.tree.has_changed() {
            self.request_redraw(window_id);
        }
    }

    fn render(&mut self) -> Result<(), Error> {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.render()
        } else {
            Err(Error::UnitializedRenderer)
        }
    }
}

fn create_window(event_loop: &winit::event_loop::ActiveEventLoop) -> Arc<Window> {
    let window = event_loop.create_window(Default::default()).unwrap();
    window.set_title("My App");
    window.set_transparent(true);
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
            self.renderer = Some(Renderer::new(window.clone(), &mut self.tree, view_fn));
            self.window.insert(window.id(), window);
        }
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
                    Err(Error::SurfaceRendering(surface_err)) => {
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
                self.tree.handle_click(&mut self.cursor, gfx);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let renderer = self.renderer.as_mut().unwrap();
                self.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                self.tree.detect_hover(&mut self.cursor, &renderer.gfx);
                self.tree.handle_hover(&mut self.cursor, &mut renderer.gfx);
            }
            _ => {}
        }
        self.detect_update(window_id);
    }
}

