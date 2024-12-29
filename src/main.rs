mod error;

use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    window::Window
};

use error::Error;

fn cast_slice<A: Sized, B: Sized>(p: &[A]) -> Result<&[B], Error> {
    if align_of::<B>() > align_of::<A>()
        && (p.as_ptr() as *const () as usize) % align_of::<B>() != 0 {
        return Err(Error::PointersHaveDifferentAlignmnet);
    }
    unsafe {
        let len = size_of_val::<[A]>(p) / size_of::<B>();
        Ok(core::slice::from_raw_parts(p.as_ptr() as *const B, len))
    }
}

const SHADER: &str = r"
    struct VertexInput {
        @location(0) position: vec3<f32>,
        @location(1) color: vec3<f32>,
    };

    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) color: vec3<f32>,
    };

    @vertex
    fn vs_main(input: VertexInput) -> VertexOutput {
        var out: VertexOutput;
        out.color = input.color;
        out.position = vec4<f32>(input.position, 1.0);
        return out;
    }

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return vec4<f32>(in.color, 1.0);
    }
";

#[derive(Debug, Clone, Copy)]
struct Vector3<T> {
    x: T,
    y: T,
    z: T
}

impl PartialEq for Vector3<u32> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.z == other.z
    }
}

#[derive(Debug, Clone, Copy)]
struct Rgb<T> {
    r: T,
    g: T,
    b: T,
}

impl From<Rgb<u8>> for Rgb<f32> {
    fn from(val: Rgb<u8>) -> Self {
        Self {
            r: val.r as f32 / u8::MAX as f32,
            g: val.g as f32 / u8::MAX as f32,
            b: val.b as f32 / u8::MAX as f32,
        }
    }
}

impl PartialEq for Rgb<u8> {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: Vector3<f32>,
    color: Rgb<f32>,
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

fn _cos(x: f32, y: f32) -> f32 {
    let hyp = (x*x + y*y).sqrt();
    (x / hyp).abs()
}

struct Triangle {
    pos: Vector3<u32>,
    width: u32,
    height: u32,
    color: Rgb<u8>,
}

impl Triangle {
    const INDICES: [u16; 3] = [0, 1, 2];

    fn new(pos: Vector3<u32>, width: u32, height: u32, color: Rgb<u8>) -> Self {
        Self { pos, width, height, color }
    }

    fn data(&self, window_size: winit::dpi::PhysicalSize<u32>) -> Vec<Vertex> {
        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);
        
        let width = self.width as f32 / window_size.width as f32;
        let height = -(self.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let t = Vector3 { x: x_pos + x_center, y: y_pos, z: self.pos.z as _ };
        let l = Vector3 { x: x_pos, y: y_pos + height, z: self.pos.z as _ };
        let r = Vector3 { x: x_pos + width, y: y_pos + height, z: self.pos.z as _ };

        [
            Vertex { position: t, color: self.color.into() },
            Vertex { position: l, color: self.color.into() },
            Vertex { position: r, color: self.color.into() },
        ].to_vec()
    }

    fn is_hovered(&self,
        mouse: &Cursor,
        window_size: winit::dpi::PhysicalSize<u32>
    ) -> bool {
        let width = self.width as f32 / window_size.width as f32;
        let height = -(self.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let x_pos = -1.0 + (self.pos.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.pos.y as f32 / window_size.height as f32);

        let x_mouse = ((mouse.position.x / window_size.width as f32) - 0.5) * 2.0;
        let y_mouse = (0.5 - (mouse.position.y / window_size.height as f32)) * 2.0;

        let mouse_tan = tan(x_pos + x_center - x_mouse, y_pos - y_mouse);
        let triangle_tan = tan(x_center, height);

        (y_pos + height..y_pos).contains(&y_mouse)
            && (x_pos..x_pos + width).contains(&x_mouse)
            && mouse_tan >= triangle_tan
    }

    fn set_color<F: FnMut(&mut Rgb<u8>)>(&mut self, mut f: F) {
        f(&mut self.color);
    }

    fn set_position(
        &mut self,
        mouse: &Cursor,
    ) {
        let delta_x = mouse.position.x - mouse.click.cur.x;
        let delta_y = mouse.position.y - mouse.click.cur.y;

        self.pos.x = (mouse.click.obj.x as f32 + delta_x * 2.) as u32;
        self.pos.y = (mouse.click.obj.y as f32 + delta_y * 2.) as u32;
    }
}

struct Buffer {
    v: wgpu::Buffer,
    i: wgpu::Buffer,
}

impl Buffer {
    fn new(device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<u16>) -> Result<Self, Error> {
        let v = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(&vertices)?,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let i = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: cast_slice(&indices)?,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        Ok(Self { v, i })
    }
}

struct Pipeline {
    _shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"), source: wgpu::ShaderSource::Wgsl(SHADER.into())
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });


        Self {
            _shader: shader,
            pipeline,
        }
    }
}

struct GfxRenderer<'a> {
    gpu: GpuResources<'a>,
    pipeline: Pipeline,
    buffer: Buffer,
}

impl<'a> GfxRenderer<'a> {
    fn new(gpu: GpuResources<'a>, layouts: &Triangle) -> Result<Self, Error> {
        let pipeline = Pipeline::new(&gpu.device, gpu.config.format);
        let vtx = layouts.data(gpu.size());
        let idx = Triangle::INDICES.to_vec();
        let buffer = Buffer::new(&gpu.device, vtx, idx)?;

        Ok(Self {
            gpu,
            pipeline,
            buffer,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu.configure();
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.gpu.queue.write_buffer(
            &self.buffer.v,
            0,
            data,
        );
    }

    fn render(&mut self) -> Result<(), Error> {
        let output = self.gpu.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        draw(&mut encoder, &view, &self.pipeline, &self.buffer);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn draw(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    pipeline: &Pipeline,
    buffer: &Buffer,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.,
                }),
                store: wgpu::StoreOp::Store,
            }
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    pass.set_pipeline(&pipeline.pipeline);
    pass.set_vertex_buffer(0, buffer.v.slice(..));
    pass.set_index_buffer(buffer.i.slice(..), wgpu::IndexFormat::Uint16);
    pass.draw_indexed(0..Triangle::INDICES.len() as u32, 0, 0..1);
}

struct GpuResources<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    id: winit::window::WindowId,
}

impl<'a> GpuResources<'a> {
    fn request(window: &'a Window) -> Result<Self, Error> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }).await.ok_or(Error::NoAdapterFound)?;
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            }, None).await?;

            Ok::<(wgpu::Adapter, wgpu::Device, wgpu::Queue), Error>((adapter, device, queue))
        })?;

        let surface_capabilites = surface.get_capabilities(&adapter);
        let format = surface_capabilites
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilites.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilites.present_modes[0],
            alpha_mode: surface_capabilites.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            id: window.id(),
        })
    }

    fn configure(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        let width = self.config.width;
        let height = self.config.height;
        winit::dpi::PhysicalSize::new(width, height)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MouseState {
    action: winit::event::ElementState,
    button: winit::event::MouseButton,
}

#[derive(Debug, Clone, Copy)]
struct MouseClick {
    cur: Vector3<f32>,
    obj: Vector3<u32>,
}

#[derive(Debug)]
struct Cursor {
    position: winit::dpi::PhysicalPosition<f32>,
    state: MouseState,
    click: MouseClick,
}

impl Cursor {
    fn set_state(&mut self,
        action: winit::event::ElementState,
        button: winit::event::MouseButton
    ) {
        self.state = MouseState { action, button };
    }
}

struct App<'a> {
    gfx: Option<GfxRenderer<'a>>,
    window: Option<Window>,
    // later change this into Vec<Widget>
    layouts: Triangle,
    cursor: Cursor,
}

impl App<'_> {
    fn new(layouts: Triangle) -> Self {
        Self {
            gfx: None,
            window: None,
            layouts,
            cursor: Cursor {
                position: winit::dpi::PhysicalPosition::new(0., 0.),
                state: MouseState {
                    action: winit::event::ElementState::Released,
                    button: winit::event::MouseButton::Left,
                },
                click: MouseClick {
                    cur: Vector3 { x: 0., y: 0., z: 0. },
                    obj: Vector3 { x: 0, y: 0, z: 0 },
                }
            },
        }
    }

    fn request_gpu(&self) -> Result<GpuResources, Error> {
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
                    let vtx = self.layouts.data(window.inner_size());
                    let data = cast_slice(&vtx).unwrap();
                    gfx.update(data);

                    match gfx.render() {
                        Ok(_) => {},
                        Err(Error::SurfaceRendering(surface_err)) => {
                            match surface_err {
                                wgpu::SurfaceError::Outdated
                                | wgpu::SurfaceError::Lost => gfx.resize(window.inner_size()),
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
                    gfx.resize(new_size);
                }
                WindowEvent::MouseInput { state: action, button, .. } => {
                    self.cursor.set_state(action, button);

                    let cur_color = self.layouts.color;
                    if self.layouts.is_hovered(&self.cursor, window.inner_size()) {
                        match self.cursor.state.action {
                            winit::event::ElementState::Pressed => {
                                self.layouts.set_color(|c| {
                                    *c = Rgb { r: 0, g: 255, b: 0 };
                                });
                                self.cursor.click.cur.x = self.cursor.position.x;
                                self.cursor.click.cur.y = self.cursor.position.y;
                                self.cursor.click.obj.x = self.layouts.pos.x;
                                self.cursor.click.obj.y = self.layouts.pos.y;
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
                    self.cursor.position = position.cast();

                    let cur_color = self.layouts.color;
                    let cur_pos = self.layouts.pos;

                    if self.layouts.is_hovered(&self.cursor, window.inner_size()) {
                        match self.cursor.state.action {
                            winit::event::ElementState::Pressed => {
                                self.layouts.set_color(|c| {
                                    *c = Rgb { r: 0, g: 255, b: 0 };
                                });
                                self.layouts.set_position(&self.cursor);
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

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let triangle = Triangle::new(Vector3 { x: 0, y: 0, z: 0 }, 1500, 1000, Rgb { r: 255, g: 0, b: 0 });
    let mut app = App::new(triangle);
    event_loop.run_app(&mut app)?;

    Ok(())
}
