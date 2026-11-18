import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Body {
    color: vec4<f32>,
    light: vec4<f32>,
    ember: vec4<f32>,
    axis: vec4<f32>,
    tone: vec4<f32>,
    storm: vec4<f32>,
    spot: vec4<f32>,
    rings: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> body: Body;

const NIGHT_SIDE: f32 = 0.03;
const SHADES: f32 = 4.0;
const LIMB: f32 = 0.55;
const TAU: f32 = 6.2831853;
const RINGLETS: f32 = 5.0;
const RING_DEPTH: f32 = 0.6;
const RING_EDGE: f32 = 0.04;

fn ring_cover(u: f32, inner: f32, density: f32) -> f32 {
    if (u < inner || u > 1.0) {
        return 0.0;
    }
    let t = (u - inner) / max(1.0 - inner, 0.0001);
    let edge = smoothstep(inner, inner + RING_EDGE, u) * (1.0 - smoothstep(1.0 - RING_EDGE, 1.0, u));
    let wave = TAU * RINGLETS * t + 1.7 * sin(TAU * 1.3 * t);
    return density * edge * (1.0 - RING_DEPTH * (0.5 + 0.5 * cos(wave)));
}

fn ring_shade(n: vec3<f32>, l: vec3<f32>) -> f32 {
    let outer = body.rings.y;
    let axis = body.axis.xyz;
    let height = dot(n, axis);
    let rise = dot(l, axis);
    if (outer <= 0.0 || height * rise >= 0.0) {
        return 1.0;
    }
    let hit = n - l * (height / rise);
    return 1.0 - ring_cover(length(hit) / outer, body.rings.x / outer, body.rings.z);
}

fn band(lat: f32) -> f32 {
    let k = body.axis.w;
    let wave = sin(lat * k + body.tone.w + 0.8 * sin(lat * k * 0.43 + 1.3));
    return smoothstep(-0.35, 0.35, wave);
}

fn surface(n: vec3<f32>) -> vec3<f32> {
    if (body.axis.w <= 0.0) {
        return body.color.rgb;
    }
    let lat = asin(clamp(dot(n, body.axis.xyz), -1.0, 1.0));
    let banded = mix(body.color.rgb, body.tone.rgb, band(lat));
    let size = body.storm.w;
    let spot = smoothstep(size, mix(size, 1.0, 0.5), dot(n, body.storm.xyz));
    return mix(banded, body.spot.rgb, spot);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position - in.world_position.xyz);
    let facing = max(dot(n, v), 0.0);
    let glow = body.color.rgb * body.color.w * (0.65 + 0.35 * facing);
    let l = body.light.xyz;
    let day = smoothstep(-0.08, 0.35, dot(n, l));
    let shade = floor(day * ring_shade(n, l) * body.light.w * SHADES + 0.5) / SHADES;
    let limb = select(1.0, mix(LIMB, 1.0, sqrt(facing)), body.axis.w > 0.0);
    let lit = surface(n) * mix(NIGHT_SIDE, 1.0, shade) * limb + body.ember.rgb;
    return vec4<f32>(mix(lit, glow, step(0.001, body.color.w)), 1.0);
}
