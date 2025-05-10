use shared::{Matrix4x4, Rgba, Size, Vector2};

use super::shader::SHADER;
use super::gpu::Gpu;
use super::gfx::Gfx;
use super::element::{CornerRadius, Element, Shape};

pub(crate) trait RenderComponentSource {
    fn fill_color(&self) -> Rgba<f32>;
    fn stroke_color(&self) -> Rgba<f32>;
    fn size(&self) -> Size<f32>;
    fn corners(&self) -> CornerRadius;
    fn shape(&self) -> Shape;
    fn rotation(&self) -> f32;
    fn stroke_width(&self) -> f32;
    fn texture_id(&self) -> i32;
    fn transform(&self, window_size: Size<f32>) -> Matrix4x4;

    fn element(&self) -> Element {
        Element::new(
            self.fill_color(),
            self.stroke_color(),
            self.corners(),
            self.size(),
            self.shape(),
            self.rotation(),
            self.stroke_width(),
            self.texture_id(),
        )
    }
}

pub(crate) trait TextureDataSource {
    fn data(&self) -> &[u8];
    fn dimensions(&self) -> Size<u32>;
}

pub(crate) trait IntoRenderSource {
    type RenderComponentSource: RenderComponentSource;
    type TetureDataSource: TextureDataSource;

    fn render_components_source(&self) -> &[Self::RenderComponentSource];
    fn texture_data_source(&self) -> &[Self::TetureDataSource];

    fn register(&self, gpu: &Gpu, gfx: &mut Gfx) {
        self.render_components_source().iter().skip(1).for_each(|rcs| {
            let maybe_pixel = if rcs.texture_id() >= 0 {
                Some(&self.texture_data_source()[rcs.texture_id() as usize])
            } else {
                None
            };
            gfx.register(gpu, maybe_pixel, rcs);
        });
    }
}

// .....................................

#[derive(Debug, Clone)]
pub(crate) struct Indices<'a>(&'a [u32]);

impl std::ops::Deref for Indices<'_> {
    type Target = [u32];
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Indices<'_> {
    pub(crate) const fn new() -> Self {
        Self(&[0, 1, 2, 2, 3, 0])
    }

    // const fn three() -> Self {
    //     Self(&[0, 1, 2])
    // }
}

// ....................................

#[derive(Clone, Copy)]
pub(crate) struct Vertex {
    _pos: Vector2<f32>,
    _uv: Vector2<f32>,
}

// ....................................

pub(crate) struct Vertices([Vertex; 4]);

impl std::ops::Deref for Vertices {
    type Target = [Vertex];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for Vertices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl Vertices {
    const VERTICES: Self = Self ([
        Vertex { _pos: Vector2::new( -1.0,  1.0 ), _uv: Vector2::new( 0.0, 0.0 ) },
        Vertex { _pos: Vector2::new( -1.0, -1.0 ), _uv: Vector2::new( 0.0, 1.0 ) },
        Vertex { _pos: Vector2::new(  1.0, -1.0 ), _uv: Vector2::new( 1.0, 1.0 ) },
        Vertex { _pos: Vector2::new(  1.0,  1.0 ), _uv: Vector2::new( 1.0, 0.0 ) },
    ]);

    pub(crate) fn new() -> Self {
        // let s = size / DEFAULT_SCALER / 2.;
        // Self::VERTICES.adjust(s.width(), s.height())
        Self::VERTICES
    }

    pub(crate) fn as_slice(&self) -> &[Vertex] {
        self.0.as_slice()
    }

    // fn adjust(mut self, sw: f32, sh: f32) -> Self {
    //     self.iter_mut().for_each(|vertex| {
    //         vertex._pos.mul_x(sw);
    //         vertex._pos.mul_y(sh);
    //     });
    //     self
    // }
}

impl std::fmt::Debug for Vertices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let len = self.0.len();
        for i in 0..len {
            let pos = self.0[i]._pos;
            if i == len - 1 {
                s.push_str(format!("{i}: {pos:?}").as_str());
            } else {
                s.push_str(format!("{i}: {pos:?}\n").as_str());
            }
        }
        write!(f, "{s}")
    }
}

pub(crate) fn cast_slice<SRC: Sized, DST: Sized>(src: &[SRC]) -> &[DST] {
    let len = size_of_val(src);
    unsafe { core::slice::from_raw_parts(src.as_ptr() as *const DST, len) }
}

pub(crate) fn create_pipeline(
    gpu: &Gpu,
    buffers: &[wgpu::VertexBufferLayout<'_>],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let device = &gpu.device;
    let format = gpu.config.format;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"), source: wgpu::ShaderSource::Wgsl(SHADER.into())
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });
    let blend_comp = wgpu::BlendComponent {
        operation: wgpu::BlendOperation::Add,
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("render pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers,
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        },
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
                blend: Some(wgpu::BlendState {
                    color: blend_comp,
                    alpha: blend_comp,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        depth_stencil: None,
        multiview: None,
        cache: None,
    })
}
