pub(crate) fn create_shader<'a>(input: &'a [&str]) -> std::borrow::Cow<'a, str> {
    let mut s = String::new();
    input.iter().for_each(|shader| s.push_str(shader));
    s.into()
}

pub const VERTEX: &str = r"
@group(0) @binding(0) var<uniform> screen_t: mat3x2f;
@group(0) @binding(1) var<uniform> screen_s: vec2<f32>;

struct Radius {
    top_left: f32,
    bot_left: f32,
    bot_right: f32,
    top_right: f32,
};

fn scale_radius(r: Radius, ew: f32) -> Radius {
    var ret: Radius;
    ret.top_left = (r.top_left * ew / (100.0 * 2.0)) / screen_s.x;
    ret.bot_left = (r.bot_left * ew / (100.0 * 2.0)) / screen_s.x;
    ret.bot_right = (r.bot_right * ew / (100.0 * 2.0)) / screen_s.x;
    ret.top_right = (r.top_right * ew / (100.0 * 2.0)) / screen_s.x;
    return ret;
}

struct Element {
    color: vec4<f32>,
    stroke_color: vec4<f32>,
    radius: Radius,
    size: vec2<f32>,
    shape: u32,
    rotate: f32,
    stroke_width: f32,
    image_id: i32,
    atlas_id: i32,
    transform_id: u32,
};

@group(1) @binding(0) var<storage> elements: array<Element>;
@group(1) @binding(1) var<storage> transforms: array<mat3x2<f32>>;

fn rotate(r: f32, pos: vec2<f32>) -> vec2<f32> {
    let rotation = mat2x2<f32>(
        cos(r), -sin(r),
        sin(r),  cos(r),
    );
    return rotation * pos;
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
};

@vertex
fn vs_main(vertex: VertexInput) -> FragmentPayload {
    let element = elements[vertex.id];
    let element_t = transforms[element.transform_id];

    var pos = vertex.pos;
    if element.rotate != 0.0 {
        pos = rotate(element.rotate, vertex.pos);
    }

    let t_pos = element_t * vec3f(pos, 1.0);
    let s_pos = screen_t * vec3f(t_pos, 1.0);

    var out: FragmentPayload;
    out.uv = select(vertex.uv * 2 - 1, vertex.uv, element.image_id > -1 || element.atlas_id > -1);
    out.index = vertex.id;
    out.position = vec4f(s_pos, 0.0, 1.0);
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

fn sdf(uv: vec2<f32>, element: Element, stroke_width: f32) -> f32 {
    let size = element.size / screen_s;

    switch element.shape {
        case 0u: {
            let p = uv * size.x;
            let r = size.x - stroke_width;
            return sdCircle(p, r);
        }
        case 1u: {
            let p = uv * size;
            let b = size - stroke_width;
            return sdRect(p, b);
        }
        case 2u: {
            let p = uv * size;
            let b = size - stroke_width;
            let r = scale_radius(element.radius, element.size.x);
            return sdRoundedRect(p, b, r);
        }
        default: { return -1.0; }
    }
}
";

pub const FRAGMENT: &str = r"
@group(2) @binding(0) var t: texture_2d<f32>;
@group(3) @binding(0) var s: sampler;

struct Stroke {
    width: f32,
    color: vec4f,
};

fn get_stroke(element: Element) -> Stroke {
    var stroke: Stroke;
    stroke.color = element.stroke_color;
    stroke.width = element.stroke_width / screen_s.x;
    if element.stroke_width == 0 {
        stroke.color = element.color;
        stroke.width = 5.0 / screen_s.x;
    }
    return stroke;
}

@fragment
fn fs_main(in: FragmentPayload) -> @location(0) vec4<f32> {
    let element = elements[in.index];

    if element.image_id > -1 || element.atlas_id > -1 { return textureSample(t, s, in.uv); }

    let stroke = get_stroke(element);
    let sdf = sdf(in.uv, element, stroke.width);
    let fill = select(vec4f(0.0), element.color, sdf < 0.0);
    let blend = 1.0 - smoothstep(0.0, stroke.width, abs(sdf));

    return mix(fill, stroke.color, blend);
}
";
