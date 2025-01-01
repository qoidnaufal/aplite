use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};
use std::cell::RefCell;

use crate::{
    color::Rgb,
    error::Error,
    gpu::GpuResources,
    layout::Triangle,
    renderer::GfxRenderer,
    types::{cast_slice, Size, Vector2, Vector3}
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub action: winit::event::ElementState,
    pub button: winit::event::MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseClick {
    pub cur: Vector2<f32>,
    pub obj: Vector3<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub position: Vector2<f32>,
    pub state: MouseState,
    pub click: MouseClick,
}

impl Cursor {
    fn new() -> Self {
        Self {
            position: Vector2::new(0., 0.),
            state: MouseState {
                action: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Left,
            },
            click: MouseClick {
                cur: Vector2::new(0., 0.),
                obj: Vector3::new(0, 0, 0),
            }
        }
    }

    pub fn set_state(&mut self,
        action: winit::event::ElementState,
        button: winit::event::MouseButton
    ) {
        self.state = MouseState { action, button };
    }
}

pub struct App<'a> {
    pub gfx: Option<GfxRenderer<'a>>,
    pub window: Option<Window>,
    // later change this into Vec<Widget>
    pub layouts: Triangle,
}

impl App<'_> {
    pub fn new(layouts: Triangle) -> Self {
        Self {
            gfx: None,
            window: None,
            layouts,
        }
    }

    pub fn request_gpu(&self) -> Result<GpuResources, Error> {
        let gpu = GpuResources::request(self.window.as_ref().unwrap())?;
        gpu.configure();
        Ok(gpu)
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        self.window = Some(window);

        let gpu = self.request_gpu().unwrap();
        let gfx = GfxRenderer::new(gpu, &self.layouts).unwrap();
        let gfx: GfxRenderer<'a> = unsafe { std::mem::transmute(gfx) };
        self.gfx = Some(gfx);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
        let Some(ref window) = self.window else { return };
        let Some(ref mut gfx) = self.gfx else { return };

        if gfx.gpu.id == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    // println!("redraw");
                    let vtx = self.layouts.data();
                    let data = cast_slice(&vtx).unwrap();
                    gfx.update(data);

                    match gfx.render() {
                        Ok(_) => {},
                        Err(Error::SurfaceRendering(surface_err)) => {
                            match surface_err {
                                wgpu::SurfaceError::Outdated
                                | wgpu::SurfaceError::Lost => gfx.resize(),
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
                    gfx.resize();
                }
                WindowEvent::MouseInput { state: action, button, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.cursor.set_state(action, button));

                    let cur_color = self.layouts.color;
                    if self.layouts.is_hovered() {
                        match CONTEXT.with_borrow(|ctx| ctx.cursor.state.action) {
                            winit::event::ElementState::Pressed => {
                                self.layouts.set_color(|c| {
                                    *c = Rgb { r: 0, g: 255, b: 0 };
                                });
                                CONTEXT.with_borrow_mut(|ctx| {
                                    ctx.cursor.click.cur.x = ctx.cursor.position.x;
                                    ctx.cursor.click.cur.y = ctx.cursor.position.y;
                                    ctx.cursor.click.obj.x = self.layouts.pos.x;
                                    ctx.cursor.click.obj.y = self.layouts.pos.y;
                                });
                            },
                            winit::event::ElementState::Released => {
                                self.layouts.set_color(|c| {
                                    *c = Rgb { r: 0, g: 0, b: 255 };
                                });
                            },
                        }
                    }
                    if cur_color != self.layouts.color {
                        window.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.cursor.position = Vector2::from(position.cast()));

                    let cur_color = self.layouts.color;
                    let cur_pos = self.layouts.pos;

                    if self.layouts.is_hovered() {
                        match CONTEXT.with_borrow(|ctx| ctx.cursor.state.action) {
                            winit::event::ElementState::Pressed => {
                                self.layouts.set_color(|c| {
                                    *c = Rgb { r: 0, g: 255, b: 0 };
                                });
                                self.layouts.set_position();
                            },
                            winit::event::ElementState::Released => {
                                self.layouts.set_color(|c| {
                                    *c = Rgb { r: 0, g: 0, b: 255 };
                                });
                            },
                        }
                    } else {
                        self.layouts.set_color(|c| {
                            *c = Rgb { r: 255, g: 0, b: 0 };
                        });
                    }
                    if cur_color != self.layouts.color || cur_pos != self.layouts.pos {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
}

