pub const SHADER: &str = r"
    struct VertexOutput {
        @builtin(position) position: vec4<f32>,
        @location(0) uv: vec2<f32>,
        @location(1) @interpolate(flat) s_idx: u32,
    };

    struct Uniform {
        screen_transform: mat4x4<f32>,
    };

    @group(0) @binding(0) var<uniform> uniform: Uniform;

    struct Shape {
        pos: vec2<u32>,
        dimensions: vec2<u32>,
        color: vec3<f32>,
        kind: u32,
        transform: u32,
    };

    @group(1) @binding(0) var<storage> shapes: array<Shape>;

    @group(1) @binding(1) var<storage> transforms: array<mat4x4<f32>>;

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
    fn vs_main(
        @builtin(vertex_index) v_idx: u32,
        @builtin(instance_index) i_idx: u32,
    ) -> VertexOutput {
        let shape = shapes[i_idx];
        let screen_transform = uniform.screen_transform;
        let shape_transform = mat4x4<f32>(transforms[shape.transform]);
        let verts = vec4<f32>(vertices[v_idx], 1.0, 1.0);

        var out: VertexOutput;
        out.uv = uv_table[v_idx];
        out.s_idx = i_idx;
        out.position = vec4<f32>(screen_transform * shape_transform * verts);
        return out;
    }

    @group(2) @binding(0) var t: texture_2d<f32>;
    @group(0) @binding(1) var s: sampler;

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        let shape = shapes[in.s_idx];
        let color_mask = vec4<f32>(shape.color, 1.0);
        let texture_mask = textureSample(t, s, in.uv);
        return select(color_mask, texture_mask, shape.kind != 2);
        // return vec4<f32>(color, 1.0);
        // return textureSample(t, s, in.uv);
    }
";
