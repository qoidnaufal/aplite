use std::cell::RefCell;

use winit::window::Window;
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use math::{Size, Vector2};

use crate::view::{NodeId, View};
use crate::renderer::Renderer;
use crate::storage::WidgetsStorage;
use crate::renderer::Gpu;
use crate::error::Error;

thread_local! {
    pub static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub cursor: Cursor,
    pub window_size: Size<u32>,
}

impl Context {
    fn new() -> Self {
        Self {
            cursor: Cursor::new(),
            window_size: Size::new(0, 0),
        }
    }

    fn set_click_state(&mut self, action: MouseAction, button: MouseButton) {
        self.cursor.set_state(action, button);

        match (self.cursor.state.action, self.cursor.state.button) {
            (MouseAction::Pressed, MouseButton::Left) => {
                self.cursor.click.obj = self.cursor.hover.curr;
                self.cursor.click.pos = self.cursor.hover.pos;
            },
            (MouseAction::Released, MouseButton::Left) => self.cursor.click.obj = None,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Pressed,
    Released,
}

impl From<winit::event::ElementState> for MouseAction {
    fn from(value: winit::event::ElementState) -> Self {
        match value {
            winit::event::ElementState::Pressed => Self::Pressed,
            winit::event::ElementState::Released => Self::Released,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(value: winit::event::MouseButton) -> Self {
        match value {
            winit::event::MouseButton::Left => Self::Left,
            winit::event::MouseButton::Right => Self::Right,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Back => Self::Back,
            winit::event::MouseButton::Forward => Self::Forward,
            winit::event::MouseButton::Other(n) => Self::Other(n),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub action: MouseAction,
    pub button: MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseClick {
    pub pos: Vector2<f32>,
    pub obj: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseHover {
    pub pos: Vector2<f32>,
    pub curr: Option<NodeId>,
    pub prev: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    pub hover: MouseHover,
    pub state: MouseState,
    pub click: MouseClick,
}

impl Cursor {
    fn new() -> Self {
        Self {
            hover: MouseHover {
                pos: Vector2::new(),
                curr: None,
                prev: None,
            },
            state: MouseState {
                action: MouseAction::Released,
                button: MouseButton::Left,
            },
            click: MouseClick {
                pos: Vector2::new(),
                obj: None,
            },
        }
    }

    pub fn set_state(&mut self,
        action: MouseAction,
        button: MouseButton
    ) {
        self.state = MouseState { action, button };
    }

    pub fn is_dragging(&self, hover_id: NodeId) -> bool {
        self.click.obj.is_some_and(|click_id| click_id == hover_id)
            && self.hover.pos != self.click.pos
    }

    pub fn is_hovering_same_obj(&self) -> bool {
        self.hover.curr == self.hover.prev
    }
}

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
        eprintln!("average render time: {avg:?}");
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
    pub widgets: WidgetsStorage,
    initial_size: Size<u32>,
    stats: Stats<50>,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            window: None,
            widgets: WidgetsStorage::new(),
            initial_size: Size::new(0, 0),
            stats: Stats::new(),
        }
    }

    fn request_gpu(&self) -> Result<Gpu, Error> {
        let gpu = Gpu::request(self.window.as_ref().unwrap())?;
        gpu.configure();
        Ok(gpu)
    }

    fn request_redraw(&self) {
        self.window.as_ref().unwrap().request_redraw();
    }

    fn resize(&mut self) {
        self.renderer.as_mut().unwrap().resize(&mut self.widgets);
    }

    fn id(&self) -> winit::window::WindowId {
        let gfx = self.renderer.as_ref().unwrap();
        gfx.gpu.id
    }

    fn detect_hover(&self) {
        self.widgets.detect_hover();
    }

    fn update(&mut self) {
        while let Some(ref change_id) = self.widgets.changed_ids.pop() {
            let shape = self.widgets.shapes.get(change_id).unwrap();
            self.renderer.as_mut().unwrap().update(change_id, shape);
        }
    }

    fn render(&mut self) -> Result<(), Error> {
        self.renderer.as_mut().unwrap().render(&self.widgets.nodes)
    }

    pub fn add_widget(&mut self, node: impl View) -> &mut Self {
        self.widgets.insert(node);
        self
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        window.set_title("My App");

        let size = window.inner_size();
        self.initial_size = Size::new(size.width, size.height);
        CONTEXT.with_borrow_mut(|ctx| ctx.window_size = self.initial_size);
        self.window = Some(window);
        self.widgets.compute_layout();

        let gpu = self.request_gpu().unwrap();
        let renderer: Renderer<'a> = unsafe { std::mem::transmute(Renderer::new(gpu, &self.widgets)) };
        self.renderer = Some(renderer);

        eprintln!("{:#?}", self.widgets);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
        if self.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
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
                                    self.resize();
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
                    let elapsed = start.elapsed();
                    // eprintln!("{elapsed:?}");
                    self.stats.push(elapsed);
                }
                WindowEvent::Resized(new_size) => {
                    CONTEXT.with_borrow_mut(|ctx| {
                        ctx.window_size = Size::new(new_size.width, new_size.height);
                    });
                    self.resize();
                }
                WindowEvent::MouseInput { state: action, button, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.set_click_state(action.into(), button.into()));

                    self.widgets.handle_click();
                    if self.widgets.has_changed() {
                        self.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let p: winit::dpi::PhysicalPosition<f32> = position.cast();
                    CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.pos = Vector2::from((p.x, p.y)));
                    self.detect_hover();

                    self.widgets.handle_hover();
                    if self.widgets.has_changed() {
                        self.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
}

