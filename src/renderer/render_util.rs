use shared::{Matrix4x4, Size, Rgba};

use super::{
    CornerRadius,
    Element,
    Gfx,
    Gpu,
    Shape,
    SHADER,
};

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
