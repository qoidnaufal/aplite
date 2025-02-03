use std::collections::HashMap;
use std::mem::MaybeUninit;

use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use util::{Size, Vector2};

use crate::context::Cursor;
use crate::renderer::Renderer;
use crate::storage::WidgetStorage;
use crate::renderer::Gpu;
use crate::error::Error;
use crate::IntoView;

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
        eprintln!("average update time: {avg:?}");
    }
}

pub struct App<'a> {
    pub renderer: MaybeUninit<Renderer<'a>>,
    pub storage: WidgetStorage,
    pub cursor: Cursor,
    window: HashMap<WindowId, Window>,
    stats: Stats,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            renderer: MaybeUninit::uninit(),
            window: HashMap::new(),
            storage: WidgetStorage::new(),
            cursor: Cursor::new(),
            stats: Stats::new(),
        }
    }

    fn request_redraw(&self, window_id: winit::window::WindowId) {
        if let Some(window) = self.window.get(&window_id) {
            window.request_redraw();
        }
    }

    fn resize(&mut self, size: Size<u32>) {
        unsafe { self.renderer.assume_init_mut().resize(size) }
    }

    fn update(&mut self) {
        unsafe {
            let renderer = self.renderer.assume_init_mut();
            self.storage.submit_update(renderer);
        }
    }

    fn render(&mut self) -> Result<(), Error> {
        unsafe { self.renderer.assume_init_mut().render(&self.storage.nodes) }
    }

    pub fn add_widget(&mut self, node: impl IntoView) {
        self.storage.insert(node);
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        window.set_title("My App");

        let renderer: Renderer<'a> = unsafe {
            std::mem::transmute(Renderer::new(&window, &mut self.storage))
        };
        self.window.insert(window.id(), window);
        self.renderer.write(renderer);

        // eprintln!("{:?}", self.storage.layout);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let size = unsafe { self.renderer.assume_init_mut().gpu.size() };
        match event {
            WindowEvent::CloseRequested => {
                eprintln!();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let start = std::time::Instant::now();
                self.update();
                let elapsed = start.elapsed();
                self.stats.inc(elapsed);

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
                    Err(_) => panic!()
                }
            }
            WindowEvent::Resized(new_size) => {
                self.resize(Size::new(new_size.width, new_size.height));
            }
            WindowEvent::MouseInput { state: action, button, .. } => {
                self.cursor.set_click_state(action.into(), button.into());
                self.storage.handle_click(&mut self.cursor);
                if self.storage.has_changed() {
                    self.request_redraw(window_id);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                self.storage.detect_hover(&mut self.cursor);
                self.storage.handle_hover(&mut self.cursor);
                if self.storage.has_changed() {
                    self.request_redraw(window_id);
                }
            }
            _ => {}
        }
    }
}

