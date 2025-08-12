pub(crate) fn create_shader<'a>(input: &'a [&str]) -> std::borrow::Cow<'a, str> {
    let shader = input.iter().cloned().collect::<String>();
    shader.into()
}

pub(crate) fn render<'a>() -> std::borrow::Cow<'a, str> {
    create_shader(&[VERTEX, SDF, FRAGMENT])
}

pub const VERTEX: &str = r"
@group(0) @binding(0) var<uniform> screen_t: mat3x2f;

struct Radius {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
}

fn scale_radius(r: Radius, ew: f32) -> Radius {
    var ret: Radius;
    ret.top_left = (r.top_left * ew / (100.0 * 2.0));
    ret.bot_left = (r.bot_left * ew / (100.0 * 2.0));
    ret.bot_right = (r.bot_right * ew / (100.0 * 2.0));
    ret.top_right = (r.top_right * ew / (100.0 * 2.0));
    return ret;
}

struct Element {
    background: vec4<f32>,
    border: vec4<f32>,
    radius: Radius,
    shape: u32,
    rotation: f32,
    border_width: f32,
    atlas_id: i32,
}

@group(1) @binding(0) var<storage> elements: array<Element>;
@group(1) @binding(1) var<storage> transforms: array<mat3x2<f32>>;

// scale -> rotate -> translate
fn transform_point(tx: mat3x2<f32>, rotation: f32, pos: vec2<f32>) -> vec2f {
    let sin = sin(rotation);
    let cos = cos(rotation);

    let r = mat2x2<f32>(
        cos, -sin,
        sin,  cos,
    );

    let t = mat2x2<f32>(tx[0], tx[1]);

    let s_mat = mat2x2<f32>(screen_t[0], screen_t[1]);

    return s_mat * (r * t * pos + tx[2]) + screen_t[2];
}

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) id: u32,
}

struct FragmentPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) index: u32,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> FragmentPayload {
    let element = elements[vertex.id];
    let transform = transforms[vertex.id];

    let pos = transform_point(transform, element.rotation, vertex.pos);

    var out: FragmentPayload;
    out.uv = select(vertex.uv * 2 - 1, vertex.uv, element.atlas_id > -1);
    out.index = vertex.id;
    out.position = vec4f(pos, 0.0, 1.0);
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

fn sdRoundedRect(p: vec2<f32>, b: vec2<f32>, r: Radius) -> f32 {
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
fn sdf(uv: vec2<f32>, index: u32, element: Element) -> f32 {
    let transform = transforms[index];
    let size = vec2f(transform[0].x, transform[1].y);
    let border_width = element.border_width;

    switch element.shape {
        case 0u: {
            let p = uv * size.x;
            let r = size.x - border_width;
            return sdCircle(p, r);
        }
        case 1u: {
            let p = uv * size;
            let b = size - border_width;
            return sdRect(p, b);
        }
        case 2u: {
            let p = uv * size;
            let b = size - border_width;
            let r = scale_radius(element.radius, size.x);
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

    if element.atlas_id > -1 { return textureSample(t, s, in.uv); }

    let sdf = sdf(in.uv, in.index, element);
    let blend = 1.0 - smoothstep(0.0, element.border_width, abs(sdf));

    let color = select(vec4f(0.0), element.background, sdf < 0.0);
    return mix(color, element.border, blend);
}
";

// pub const COMPUTE_SDF: &str = r"
// @compute @workgroup_size(16, 16)
// fn compute(@builtin(global_invocation_id) id: vec3u) {
// }
// ";
