pub const SHADER: &str = r"
    // struct VertexInput {
    //     @location(0) position: vec2<f32>,
    //     @location(1) uv: vec2<f32>,
    // };

    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) uv: vec2<f32>,
    };

    struct Transform {
        mat: mat4x4<f32>,
    };

    @group(0) @binding(0) var<uniform> transform: Transform;

    const vertices = array<vec2<f32>, 5>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 0.0,  1.0),
    );

    const uv_table = array<vec2<f32>, 5>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.5, 0.0),
    );

    @vertex
    fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
        var out: VertexOutput;
        out.uv = uv_table[idx];
        out.position = vec4<f32>(transform.mat * vec4<f32>(vertices[idx], 1.0, 1.0));
        return out;
    }

    @group(0) @binding(1) var t: texture_2d<f32>;
    @group(0) @binding(2) var s: sampler;

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return textureSample(t, s, in.uv);
    }
";

pub const _SDF_SHADER: &str = r"
    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) @interpolate(flat) prim_index: u32,
        @location(1) t: vec2<f32>,
        @location(2) p: vec2<f32>,
        @location(3) size: vec2<f32>,
    };

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        let fw = length(fwidth(in.t));
        let prim = prims.prims[in.prim_index];
        let paint = paints.paints[prim.paint];
        let scissor = scissors.scissors[prim.scissor];

        let mask = textureSample(glyph_atlas, samp, in.t / in.size);
        let color_mask = textureSample(color_atlas, color_samp, in.t / in.size);
    }
";
