use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::Window,
};
use std::cell::RefCell;

use crate::{
    error::Error,
    gpu::GpuResources,
    layout::Layout,
    renderer::GfxRenderer,
    types::{Size, Vector2},
    widget::{NodeId, Widget},
};

thread_local! {
    pub static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub cursor: Cursor,
    pub window_size: Size<u32>,
}

impl Context {
    fn new() -> Self {
        Self {
            cursor: Cursor::new(),
            window_size: Size::new(0, 0)
        }
    }

    fn set_click_state(&mut self, action: MouseAction, button: MouseButton) {
        self.cursor.set_state(action, button);

        match (self.cursor.state.action, self.cursor.state.button) {
            (MouseAction::Pressed, MouseButton::Left) => {
                self.cursor.click.obj = self.cursor.hover.obj;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseClick {
    pub pos: Vector2<f32>,
    pub obj: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseHover {
    pub pos: Vector2<f32>,
    pub obj: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                obj: None,
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
}

pub struct App<'a> {
    pub gfx: Option<GfxRenderer<'a>>,
    pub window: Option<Window>,
    pub layout: Layout,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            gfx: None,
            window: None,
            layout: Layout::new(),
        }
    }

    fn request_gpu(&self) -> Result<GpuResources, Error> {
        let gpu = GpuResources::request(self.window.as_ref().unwrap())?;
        gpu.configure();
        Ok(gpu)
    }

    fn request_redraw(&self) {
        self.window.as_ref().unwrap().request_redraw();
    }

    fn resize(&mut self) {
        self.gfx.as_mut().unwrap().resize();
    }

    fn id(&self) -> winit::window::WindowId {
        let gfx = self.gfx.as_ref().unwrap();
        gfx.gpu.id
    }

    fn detect_hover(&self) {
        self.layout.detect_hover();
    }

    fn update(&mut self) {
        let data = self.layout.vertices();
        self.gfx.as_mut().unwrap().update(&data);
    }

    fn render(&mut self) -> Result<(), Error> {
        self.gfx.as_mut().unwrap().render(self.layout.indices_len())
    }

    pub fn add_widget(&mut self, node: impl Widget) -> &mut Self {
        self.layout.insert(node);
        self
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        window.set_title("My App");
        CONTEXT.with_borrow_mut(|ctx| ctx.window_size = Size::from(window.inner_size()));
        self.window = Some(window);

        self.layout.calculate();

        let gpu = self.request_gpu().unwrap();
        let gfx = GfxRenderer::new(gpu, &self.layout);
        let gfx: GfxRenderer<'a> = unsafe { std::mem::transmute(gfx) };
        self.gfx = Some(gfx);
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

                    match self.render() {
                        Ok(_) => {},
                        Err(Error::SurfaceRendering(surface_err)) => {
                            match surface_err {
                                wgpu::SurfaceError::Outdated
                                | wgpu::SurfaceError::Lost => self.resize(),
                                wgpu::SurfaceError::OutOfMemory => {
                                    log::error!("Out of Memory");
                                    event_loop.exit();
                                },
                                wgpu::SurfaceError::Timeout => {
                                    log::warn!("Surface Timeout")
                                },
                            }
                        }
                        Err(_) => panic!()
                    }
                }
                WindowEvent::Resized(new_size) => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.window_size = Size::from(new_size));
                    self.resize();
                }
                WindowEvent::MouseInput { state: action, button, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.set_click_state(action.into(), button.into()));

                    unsafe { self.layout.handle_click() };
                    if self.layout.has_changed {
                        self.request_redraw();
                        self.layout.has_changed = false;
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.pos = Vector2::from(position.cast()));
                    self.detect_hover();

                    unsafe { self.layout.handle_hover() };
                    if self.layout.has_changed {
                        self.request_redraw();
                        self.layout.has_changed = false;
                    }
                }
                _ => {}
            }
        }
    }
}

