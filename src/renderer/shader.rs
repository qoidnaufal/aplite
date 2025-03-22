pub const SHADER: &str = r"
struct Screen {
    transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> screen: Screen;

struct Shape {
    color: vec4<f32>,
    texture_id: i32,
    kind: u32,
    radius: f32,
    transform: u32,
};

@group(1) @binding(0) var<storage> shapes: array<Shape>;
@group(1) @binding(1) var<storage> transforms: array<mat4x4<f32>>;

fn rotate(r: f32, pos: vec2<f32>) -> vec2<f32> {
    let rotation = mat2x2<f32>(
        cos(r), -sin(r),
        sin(r),  cos(r),
    );
    return rotation * pos;
}

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

struct Instance {
    @location(2) index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) index: u32,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    instance: Instance,
) -> VertexOutput {
    let shape = shapes[instance.index];
    let screen_transform = screen.transform;
    let shape_transform = transforms[shape.transform];
    let verts = vec4<f32>(vertices[v_idx], 0.0, 1.0);

    // let r = rotate(shape.rotate, vertices[v_idx]);
    // let verts = vec4<f32>(r, 0.0, 1.0);

    var out: VertexOutput;
    out.uv = uv_table[v_idx];
    out.index = instance.index;
    out.position = vec4<f32>(screen_transform * shape_transform * verts);
    return out;
}

@group(2) @binding(0) var t: texture_2d<f32>;
@group(2) @binding(1) var s: sampler;

fn sdCircle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdRoundedBox(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let d = abs(p) - b + vec2<f32>(r);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2f(0.0, 0.0))) - r;
}

fn sdf(uv: vec2<f32>, shape: Shape) -> f32 {
    // let transform = transforms[shape.transform];
    // let size = vec2<f32>(transform[0].x, transform[1].y);
    switch shape.kind {
        case 0u: {
            let p = uv - vec2<f32>(0.5, 0.5);
            return sdCircle(p, 0.5);
        }
        case 1u: { return 0.0; }
        case 2u: {
            let p = uv - vec2<f32>(0.5, 0.5);
            let b = vec2<f32>(0.5, 0.5);
            return sdRoundedBox(p, b, shape.radius);
        }
        case 3u: { return 0.0; }
        default: { return 0.0; }
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let shape = shapes[in.index];
    let shape_color = shape.color;

    let d = sdf(in.uv, shape);
    let sdf_color = vec4<f32>(
        shape_color.rgb - sign(d) * shape_color.rgb,
        shape_color.a - sign(d)
    );

    var color_mask: vec4<f32>;
    switch shape.kind {
        case 0u: { color_mask = sdf_color; }
        case 1u: { color_mask = shape_color; }
        case 2u: { color_mask = sdf_color; }
        case 3u: { color_mask = shape_color; }
        default: { color_mask = shape_color; }
    }

    let texture_mask = textureSample(t, s, in.uv);
    return select(color_mask, texture_mask, shape.texture_id > -1);
}
";
