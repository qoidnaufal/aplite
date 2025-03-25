pub const SHADER: &str = r"
struct Screen {
    transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> screen: Screen;

struct Radius {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
};

struct Shape {
    color: vec4<f32>,
    outline: vec4<f32>,
    radius: Radius,
    kind: u32,
    rotate: f32,
    stroke: f32,
    texture_id: i32,
    transform_id: u32,
};

@group(1) @binding(0) var<storage> shapes: array<Shape>;
@group(1) @binding(1) var<storage> transforms: array<mat4x4<f32>>;

fn rotate(r: f32, mt: mat4x4<f32>) -> mat4x4<f32> {
    let rotation = mat2x2<f32>(
        cos(r), -sin(r),
        sin(r),  cos(r),
    );
    let m2 = mat2x2(mt[0].x, mt[0].y, mt[1].x, mt[1].y);
    let mr = rotation * m2;
    return mat4x4<f32>(
        mr[0].x, mr[0].y, mt[0].z, mt[0].w,
        mr[1].x, mr[1].y, mt[1].z, mt[1].w,
        mt[2].x, mt[2].y, mt[2].z, mt[2].w,
        mt[3].x, mt[3].y, mt[3].z, mt[3].w,
    );
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
    @builtin(vertex_index) v: u32,
    instance: Instance,
) -> VertexOutput {
    let shape = shapes[instance.index];
    let screen_t = screen.transform;
    let shape_t = transforms[shape.transform_id];

    let transform = rotate(shape.rotate, shape_t);
    let verts = vec4<f32>(vertices[v], 0.0, 1.0);

    var out: VertexOutput;
    out.uv = select(uv_table[v], uv_table[v] * 2 - 1, shape.texture_id < 0);
    out.index = instance.index;
    out.position = vec4<f32>(screen_t * transform * verts);
    return out;
}

@group(2) @binding(0) var t: texture_2d<f32>;
@group(2) @binding(1) var s: sampler;

fn sdCircle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdRect(p: vec2f, b: vec2f) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2f(0.))) + min(max(d.x, d.y), 0.);
}

fn sdRoundedRect(p: vec2<f32>, b: vec2<f32>, r: Radius) -> f32 {
    var x = r.top_left;
    var y = r.bot_left;
    x = select(r.bot_right, r.top_left, p.x < 0.);
    y = select(r.top_right, r.bot_left, p.x < 0.);
    x  = select(y, x, p.y < 0.);
    let d = abs(p) - b + x;
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2f(0.0))) - x;
}

fn sdSegment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
    return length(pa - ba * h);
}

fn sdf(uv: vec2<f32>, shape: Shape) -> f32 {
    let transform = transforms[shape.transform_id];
    let size = vec2f(transform[0].x, transform[1].y);
    switch shape.kind {
        case 0u: {
            let p = uv * size.x;
            let t = size * shape.stroke;
            let r = size.x - t.x;
            return sdCircle(p, r);
        }
        case 1u: {
            let p = uv * size;
            let t = size * shape.stroke;
            let b = size - t;
            return sdRect(p, b);
        }
        case 2u: {
            let p = uv * size;
            let t = size * shape.stroke;
            let b = size - t;
            return sdRoundedRect(p, b, shape.radius);
        }
        default: { return -1.0; }
    }
}

// fn sdfColor(sdf: f32, color: vec4<f32>) -> vec4<f32> {
//     return vec4<f32>(
//         color.rgb - sign(sdf) * color.rgb,
//         color.a - sign(sdf)
//     );
// }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let shape = shapes[in.index];
    let shape_color = shape.color;
    let border = shape.outline;

    let transform = transforms[shape.transform_id];
    let size = vec2f(transform[0].x, transform[1].y);
    let st = size.x * shape.stroke;

    let sdf = sdf(in.uv, shape);
    let sdf_color = select(vec4f(0.0), shape.color, sdf < 0.0);

    let blend = 1.0 - smoothstep(0.0, st, abs(sdf));
    let color_mask = mix(sdf_color, border, select(0.0, 1.0, blend > 0.0));
    let texture_mask = textureSample(t, s, in.uv);

    return select(color_mask, texture_mask, shape.texture_id > -1);
}
";
