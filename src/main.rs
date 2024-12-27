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
    if align_of::<B>() > align_of::<A>() && (p.as_ptr() as *const () as usize) % align_of::<B>() != 0 {
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
struct Vec3<T> {
    x: T,
    y: T,
    z: T
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

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: Vec3<f32>,
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
                    offset: std::mem::size_of::<Vec3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

struct Triangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Rgb<u8>,
}

impl Triangle {
    fn new(x: u32, y: u32, width: u32, height: u32, color: Rgb<u8>) -> Self {
        Self { x, y, width, height, color }
    }

    fn is_hovered(&self,
        mouse_pos: winit::dpi::PhysicalPosition<f32>,
        window_size: winit::dpi::PhysicalSize<u32>
    ) -> bool {
        let width = self.width as f32 / window_size.width as f32;
        let height = -(self.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let x_pos = -1.0 + (self.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.y as f32 / window_size.height as f32);

        let t = Vec3 { x: x_pos + x_center, y: y_pos, z: 0.0 };
        let l = Vec3 { x: x_pos, y: y_pos + height, z: 0.0 };
        let r = Vec3 { x: x_pos + width, y: y_pos + height, z: 0.0 };

        let x_mouse = ((mouse_pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_mouse = (0.5 - (mouse_pos.y / window_size.height as f32)) * 2.0;

        // println!("mouse: {}, {} => {:?}, {:?}, {:?} => {}, {}", x_mouse, y_mouse, t, l, r, width, height);
        if (l.y..t.y).contains(&y_mouse) && (l.x..r.x).contains(&x_mouse) {
            true
        } else { false }
    }

    fn set_color(&mut self, color: Rgb<u8>) {
        self.color = color;
    }

    fn contents(&self, window_size: winit::dpi::PhysicalSize<u32>) -> BufferContents {
        let x_pos = -1.0 + (self.x as f32 / window_size.width as f32);
        let y_pos = 1.0 - (self.y as f32 / window_size.height as f32);
        
        let width = self.width as f32 / window_size.width as f32;
        let height = -(self.height as f32 / window_size.height as f32);
        let x_center = width / 2.0;

        let t = Vec3 { x: x_pos + x_center, y: y_pos, z: 0.0 };
        let l = Vec3 { x: x_pos, y: y_pos + height, z: 0.0 };
        let r = Vec3 { x: x_pos + width, y: y_pos + height, z: 0.0 };

        let vertices = [
            Vertex { position: t, color: self.color.into() },
            Vertex { position: l, color: self.color.into() },
            Vertex { position: r, color: self.color.into() },
        ].to_vec();
        let indices = vec![0, 1, 2];

        BufferContents { vertices, indices }
    }
}

struct BufferContents {
    vertices: Vec<Vertex>,
    indices: Vec<u16>
}

struct Pipeline {
    _shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    v_buffer: wgpu::Buffer,
    i_buffer: wgpu::Buffer,
    i_len: usize,
}

impl Pipeline {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, contents: BufferContents) -> Result<Self, Error> {
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
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        let v_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(&contents.vertices)?,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let i_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: cast_slice(&contents.indices)?,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });


        Ok(Self {
            _shader: shader,
            pipeline,
            v_buffer,
            i_buffer,
            i_len: contents.indices.len(),
        })
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window_id: winit::window::WindowId,
    // window: &'a Window,
    pipeline: Pipeline,
}

impl<'a> State<'a> {
    fn new(window: &'a Window, layouts: &Triangle) -> Result<Self, Error> {
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
        let surface_format = surface_capabilites
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilites.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilites.present_modes[0],
            alpha_mode: surface_capabilites.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let contents = layouts.contents(size);
        let pipeline = Pipeline::new(&device, &config, contents)?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            window_id: window.id(),
            pipeline,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self, layouts: &Triangle) -> Result<(), Error> {
        self.queue.write_buffer(
            &self.pipeline.v_buffer,
            0,
            cast_slice(&layouts.contents(self.size).vertices)?
        );

        Ok(())
    }

    fn render(&mut self) -> Result<(), Error> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render encoder") });

        clear_screen(&mut encoder, &view, &self.pipeline);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn clear_screen(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    pipeline: &Pipeline,
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
    pass.set_vertex_buffer(0, pipeline.v_buffer.slice(..));
    pass.set_index_buffer(pipeline.i_buffer.slice(..), wgpu::IndexFormat::Uint16);
    pass.draw_indexed(0..pipeline.i_len as u32, 0, 0..1);
}

struct App<'a> {
    state: Option<State<'a>>,
    window: Option<Window>,
    // later change this into Vec<Widget>
    layouts: Triangle,
}

impl<'a> App<'a> {
    fn new(layouts: Triangle) -> Self {
        Self {
            state: None,
            window: None,
            layouts,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        self.window = Some(window);

        let state = State::new(self.window.as_ref().unwrap(), &self.layouts).unwrap();
        let state: State<'a> = unsafe { std::mem::transmute(state) };
        self.state = Some(state);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
        let Some(ref window) = self.window else { return };
        let Some(ref mut state) = self.state else { return };

        if state.window_id == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    match state.update(&self.layouts) {
                        Ok(_) => {},
                        Err(_) => event_loop.exit(),
                    }
                    match state.render() {
                        Ok(_) => {},
                        Err(Error::SurfaceError(surface_err)) => {
                            match surface_err {
                                wgpu::SurfaceError::Outdated
                                | wgpu::SurfaceError::Lost => state.resize(state.size),
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
                    state.resize(new_size);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.layouts.is_hovered(position.cast(), state.size) {
                        self.layouts.set_color(Rgb { r: 0, g: 255, b: 0});
                    } else {
                        self.layouts.set_color(Rgb { r: 255, g: 0, b: 0});
                    }
                    window.request_redraw();
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

    let triangle = Triangle::new(0, 0, 1500, 1500, Rgb { r: 255, g: 0, b: 0 });
    let mut app = App::new(triangle);
    event_loop.run_app(&mut app)?;

    Ok(())
}
