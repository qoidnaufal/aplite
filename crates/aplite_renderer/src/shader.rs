pub(crate) fn create_shader<'a>(input: &'a [&str]) -> std::borrow::Cow<'a, str> {
    let shader = input.iter().cloned().collect::<String>();
    shader.into()
}

pub(crate) fn render<'a>() -> std::borrow::Cow<'a, str> {
    create_shader(&[VERTEX, SDF, FRAGMENT])
}

pub const VERTEX: &str = r"
@group(0) @binding(0) var<uniform> screen_t: mat3x2f;

struct Corners {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
}

fn unpack_corners(val: u32, ew: f32) -> Corners {
    var ret: Corners;

    let top_left = f32(val >> 24);
    let bot_left = f32((val >> 16) & 0xFF);
    let bot_right = f32((val >> 8) & 0xFF);
    let top_right = f32(val & 0xFF);

    ret.top_left = (top_left * ew / (100.0 * 2.0));
    ret.bot_left = (bot_left * ew / (100.0 * 2.0));
    ret.bot_right = (bot_right * ew / (100.0 * 2.0));
    ret.top_right = (top_right * ew / (100.0 * 2.0));

    return ret;
}

fn unpack_color(val: u32) -> vec4<f32> {
    let r = f32(val >> 24) / 255.0;
    let g = f32((val >> 16) & 0xFF) / 255.0;
    let b = f32((val >> 8) & 0xFF) / 255.0;
    let a = f32(val & 0xFF) / 255.0;

    return vec4f(r, g, b, a);
}

struct Element {
    size: vec2<f32>,
    background: u32,
    border: u32,
    corners: u32,
    shape: u32,
    border_width: f32,
}

@group(1) @binding(0) var<storage> elements: array<Element>;
@group(1) @binding(1) var<storage> transforms: array<mat3x2<f32>>;

// scale -> rotate -> translate
fn transform_point(tx: mat3x2<f32>, pos: vec2<f32>) -> vec2f {
    let r = mat2x2<f32>(tx[0], tx[1]);

    let s = mat2x2<f32>(screen_t[0], screen_t[1]);

    return s * r * pos + tx[2] + screen_t[2];
}

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) id: u32,
    @location(3) atlas: i32,
}

struct FragmentPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) index: u32,
    @location(2) @interpolate(flat) atlas: i32,
}

@vertex
fn vs_main(vertex: VertexInput) -> FragmentPayload {
    let element = elements[vertex.id];
    let transform = transforms[vertex.id];

    let pos = transform_point(transform, vertex.pos);

    var out: FragmentPayload;
    out.position = vec4f(pos, 0.0, 1.0);
    out.uv = select(vertex.uv * 2 - 1, vertex.uv, vertex.atlas > -1);
    out.index = vertex.id;
    out.atlas = vertex.atlas;
    return out;
}
";

pub const SDF: &str = r"
fn sdCircle(p: vec2<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdRect(p: vec2f, b: vec2f) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2f(0.))) + min(max(d.x, d.y), 0.);
}

fn sdRoundedRect(p: vec2<f32>, b: vec2<f32>, r: Corners) -> f32 {
    var x = r.top_left;
    var y = r.bot_left;
    x = select(r.bot_left, r.bot_right, p.x > 0.);
    y = select(r.top_left, r.top_right, p.x > 0.);
    x = select(y, x, p.y > 0.);
    let d = abs(p) - b + x;
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2f(0.0))) - x;
}

fn sdSegment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
    return length(pa - ba * h);
}

// should use uv - center here
fn sdf(uv: vec2<f32>, element: Element) -> f32 {
    let border_width = element.border_width;
    let size = element.size;

    switch element.shape {
        case 0u: {
            let p = uv * size.x;
            let r = size.x - border_width;
            return sdCircle(p, r);
        }
        case 1u: {
            let p = uv * size.x;
            let b = size - border_width;
            return sdRect(p, b);
        }
        case 2u: {
            let p = uv * size;
            let b = size - border_width;
            let r = unpack_corners(element.corners, size.x);
            return sdRoundedRect(p, b, r);
        }
        default: { return -1.0; }
    }
}
";

// col = mix( col, vec3(1.0), 1.0-smoothstep(0.0,0.01,abs(d)) );

pub const FRAGMENT: &str = r"
@group(2) @binding(0) var t: texture_2d<f32>;
@group(3) @binding(0) var s: sampler;

@fragment
fn fs_main(in: FragmentPayload) -> @location(0) vec4<f32> {
    let element = elements[in.index];

    if in.atlas > -1 { return textureSample(t, s, in.uv); }

    let sdf = sdf(in.uv, element);
    let blend = 1.0 - smoothstep(0.0, element.border_width, abs(sdf));

    let background_color = unpack_color(element.background);
    let border_color = unpack_color(element.border);
    let color = select(vec4f(0.0), background_color, sdf < 0.0);
    return mix(color, border_color, blend);
}
";

// pub const COMPUTE_SDF: &str = r"
// @compute @workgroup_size(16, 16)
// fn compute(@builtin(global_invocation_id) id: vec3u) {
// }
// ";
