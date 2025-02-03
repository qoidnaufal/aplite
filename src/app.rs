use winit::window::Window;
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use math::{Size, Vector2};

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
    pub renderer: Option<Renderer<'a>>,
    pub window: Option<Window>,
    pub storage: WidgetStorage,
    pub cursor: Cursor,
    stats: Stats,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            window: None,
            storage: WidgetStorage::new(),
            cursor: Cursor::new(),
            stats: Stats::new(),
        }
    }

    fn request_redraw(&self) {
        self.window.as_ref().unwrap().request_redraw();
    }

    fn resize(&mut self, size: Size<u32>) {
        self.renderer.as_mut().unwrap().resize(size);
    }

    fn id(&self) -> winit::window::WindowId {
        let gfx = self.renderer.as_ref().unwrap();
        gfx.gpu.id
    }

    fn update(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        self.storage.submit_update(renderer);
    }

    fn render(&mut self) -> Result<(), Error> {
        self.renderer.as_mut().unwrap().render(&self.storage.nodes)
    }

    pub fn add_widget(&mut self, node: impl IntoView) {
        self.storage.insert(node);
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        window.set_title("My App");

        let gpu = Gpu::request(&window).unwrap();
        let renderer: Renderer<'a> = unsafe {
            std::mem::transmute(Renderer::new(gpu, &mut self.storage))
        };
        self.window = Some(window);
        self.renderer = Some(renderer);

        // eprintln!("{:?}", self.storage.layout);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let size = self.renderer.as_ref().unwrap().gpu.size();
        if self.id() == window_id {
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
                        self.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                    self.storage.detect_hover(&mut self.cursor);
                    self.storage.handle_hover(&mut self.cursor);
                    if self.storage.has_changed() {
                        self.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
}

