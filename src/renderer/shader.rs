pub const SHADER: &str = r"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
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

// fn shapeTransform(pos: vec2<u32>, dims: vec2<u32>, window: vec2<f32>) -> mat4x4<f32> {
//     let s = vec2<f32>(f32(dims.x) / window.x, f32(dims.y) / window.y);
//     let t = vec2<f32>(
//         (f32(pos.x) / window.x - 0.5) * 2.0,
//         (0.5 - f32(pos.y) / window.y) * 2.0
//     );

//     return mat4x4<f32>(
//         vec4<f32>(s.x, 0.0, 0.0, 0.0),
//         vec4<f32>(0.0, s.y, 0.0, 0.0),
//         vec4<f32>(0.0, 0.0, 1.0, 0.0),
//         vec4<f32>(t.x, t.y, 0.0, 1.0),
//     );
// }

struct Instance {
    @location(1) index: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    instance: Instance,
) -> VertexOutput {
    let shape = shapes[instance.index];
    let screen_transform = uniform.screen_transform;
    let shape_transform = mat4x4<f32>(transforms[instance.index]);
    let verts = vec4<f32>(vertices[v_idx], 0.0, 1.0);

    var out: VertexOutput;
    out.uv = uv_table[v_idx];
    out.position = vec4<f32>(screen_transform * shape_transform * verts);
    return out;
}

@group(2) @binding(0) var t: texture_2d<f32>;
@group(2) @binding(1) var s: sampler;

// fn sdCircle(p: vec2<f32>, r: f32) -> f32 {
//     return length(p) - r;
// }

// fn sdEquilateralTriangle(p: vec2<f32>) -> f32 {
//     let k = sqrt(3.0);
//     var q: vec2<f32> = vec2<f32>(abs(p.x) - 1.0, p.y + 1.0 / k);
//     if (q.x + k * q.y > 0.0) { q = vec2<f32>(q.x - k * q.y, -k * q.x - q.y) / 2.0; }
//     q.x = q.x - clamp(q.x, -2.0, 0.0);
//     return -length(q) * sign(q.y);
// }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let color_mask = vec4<f32>(in.color, 1.0);
    // let texture_mask = textureSample(t, s, in.uv);
    // return select(color_mask, texture_mask, in.kind == 2);

    // return vec4<f32>(color, 1.0);
    // let p = in.uv * 1.2;
    // let d = sdCircle(p, 0.7);
    // let d = sdEquilateralTriangle(p);
    // var col = vec3f(1.0) - sign(d) * vec3<f32>(0.1, 0.4, 0.7);
    // col *= 1.0 - exp(-2.0 * abs(d));
    // col *= 0.8 + 0.2 * cos(120.0 * d);
    // col = mix(col, vec3<f32>(1.0), 1.0 - smoothstep(0.0, 0.01, abs(d)));
    // return vec4<f32>(col, 1.0);

    return textureSample(t, s, in.uv);
}
";
