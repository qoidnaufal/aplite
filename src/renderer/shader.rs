pub const SHADER: &str = r"
struct Screen {
    transform: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> screen: Screen;
@group(0) @binding(1) var<uniform> window_size: vec2<u32>;

struct Radius {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
};

struct Element {
    color: vec4<f32>,
    outline: vec4<f32>,
    radius: Radius,
    kind: u32,
    rotate: f32,
    stroke: f32,
    texture_id: i32,
    transform_id: u32,
};

@group(1) @binding(0) var<storage> elements: array<Element>;
@group(1) @binding(1) var<storage> transforms: array<mat4x4<f32>>;

fn rotate(r: f32, pos: vec2<f32>) -> vec4<f32> {
    let rotation = mat2x2<f32>(
        cos(r), -sin(r),
        sin(r),  cos(r),
    );
    let xy = rotation * pos;
    return vec4<f32>(xy, 0.0, 1.0);
}

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct Instance {
    @location(2) index: u32,
};

struct FragmentPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) index: u32,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: Instance,
) -> FragmentPayload {
    let element = elements[instance.index];
    let screen_t = screen.transform;
    let element_t = transforms[element.transform_id];
    let pos = rotate(element.rotate, vertex.pos);

    var out: FragmentPayload;
    out.uv = select(vertex.uv, vertex.uv * 2 - 1, element.texture_id < 0);
    out.index = instance.index;
    out.position = screen_t * element_t * pos;
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
    x = select(r.bot_right, r.bot_left, p.x > 0.);
    y = select(r.top_right, r.top_left, p.x > 0.);
    x = select(y, x, p.y < 0.);
    let d = abs(p) - b + x;
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2f(0.0))) - x;
}

fn sdSegment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
    return length(pa - ba * h);
}

fn sdf(uv: vec2<f32>, element: Element) -> f32 {
    let transform = transforms[element.transform_id];
    let size = vec2f(transform[0].x, transform[1].y);
    let stroke = 10 * vec2f(
        element.stroke / f32(window_size.x),
        element.stroke / f32(window_size.y),
    );

    switch element.kind {
        case 0u: {
            let p = uv * size.x;
            let t = size * stroke;
            let r = size.x - t.x;
            return sdCircle(p, r);
        }
        case 1u: {
            let p = uv * size;
            let t = size * stroke;
            let b = size - t;
            return sdRect(p, b);
        }
        case 2u: {
            let p = uv * size;
            let t = size * stroke;
            let b = size - t;
            return sdRoundedRect(p, b, element.radius);
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
fn fs_main(in: FragmentPayload) -> @location(0) vec4<f32> {
    let element = elements[in.index];
    let element_color = element.color;
    let border = element.outline;

    let transform = transforms[element.transform_id];
    let size = vec2f(transform[0].x, transform[1].y);
    let stroke = element.stroke / f32(window_size.x);

    let sdf = sdf(in.uv, element);
    let fill = select(vec4f(0.0), element.color, sdf < 0.0);

    let blend = 1.0 - smoothstep(0.0, stroke, abs(sdf));
    let color_mask = mix(fill, border, select(0.0, 1.0, blend > 0.0));
    let texture_mask = textureSample(t, s, in.uv);

    return select(color_mask, texture_mask, element.texture_id > -1);
}
";
