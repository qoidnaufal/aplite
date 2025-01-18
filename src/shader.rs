pub const SHADER: &str = r"
    struct VertexInput {
        @location(0) position: vec3<f32>,
        @location(1) uv: vec2<f32>,
    };

    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) uv: vec2<f32>,
    };

    struct Uniforms {
        mat: mat4x4<f32>,
    };

    @group(0) @binding(0) var<uniform> uniforms: Uniforms;

    @vertex
    fn vs_main(input: VertexInput) -> VertexOutput {
        var out: VertexOutput;
        out.uv = input.uv;
        out.position = vec4<f32>(uniforms.mat * vec4<f32>(input.position, 1.0));
        return out;
    }

    @group(0) @binding(1) var t: texture_2d<f32>;
    @group(0) @binding(2) var s: sampler;

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return textureSample(t, s, in.uv);
    }
";
