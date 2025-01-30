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

struct Stats<const N: usize> {
    counter: usize,
    render_time: [std::time::Duration; N]
}

impl<const N: usize> Stats<N> {
    fn new() -> Self {
        Self {
            counter: 0,
            render_time: [std::time::Duration::from_micros(0); N],
        }
    }

    fn push(&mut self, d: std::time::Duration) {
        let idx = self.counter;
        self[idx] = d;
        self.counter += 1;
        self.counter %= N;
    }
}

impl<const N: usize> Drop for Stats<N> {
    fn drop(&mut self) {
        let divisor = if self.counter == 0 { N } else { self.counter };
        let avg = self.render_time[..self.counter]
            .iter()
            .sum::<std::time::Duration>() / divisor as u32;
        eprintln!("average update time: {avg:?}");
    }
}

impl<const N: usize> std::ops::Index<usize> for Stats<N> {
    type Output = std::time::Duration;
    fn index(&self, index: usize) -> &Self::Output {
        &self.render_time[index]
    }
}

impl<const N: usize> std::ops::IndexMut<usize> for Stats<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.render_time[index]
    }
}

pub struct App<'a> {
    pub renderer: Option<Renderer<'a>>,
    pub window: Option<Window>,
    pub widgets: WidgetStorage,
    pub cursor: Cursor,
    stats: Stats<50>,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            window: None,
            widgets: WidgetStorage::new(),
            cursor: Cursor::new(),
            stats: Stats::new(),
        }
    }

    fn request_redraw(&self) {
        self.window.as_ref().unwrap().request_redraw();
    }

    fn resize(&mut self, size: Size<u32>) {
        self.renderer.as_mut().unwrap().resize(&mut self.widgets, size);
    }

    fn id(&self) -> winit::window::WindowId {
        let gfx = self.renderer.as_ref().unwrap();
        gfx.gpu.id
    }

    fn update(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        self.widgets.update(renderer);
    }

    fn render(&mut self) -> Result<(), Error> {
        self.renderer.as_mut().unwrap().render(&self.widgets.nodes)
    }

    pub fn add_widget(&mut self, node: impl IntoView) {
        self.widgets.insert(node);
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        window.set_title("My App");

        let gpu = Gpu::request(&window).unwrap();
        let renderer: Renderer<'a> = unsafe {
            std::mem::transmute(Renderer::new(gpu, &mut self.widgets))
        };
        self.window = Some(window);
        self.renderer = Some(renderer);

        // eprintln!("{:?}", self.widgets.layout);
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
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    let start = std::time::Instant::now();
                    self.update();
                    let elapsed = start.elapsed();
                    self.stats.push(elapsed);

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
                    self.widgets.handle_click(&self.cursor);
                    if self.widgets.has_changed() {
                        self.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                    self.widgets.detect_hover(&mut self.cursor, size);
                    self.widgets.handle_hover(&mut self.cursor, size);
                    if self.widgets.has_changed() {
                        self.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
}

