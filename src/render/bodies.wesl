import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;
import package::render::daylight::{Place, light_at, ambient_response};

struct Body {
    color: vec4<f32>,
    light: vec4<f32>,
    ember: vec4<f32>,
    axis: vec4<f32>,
    tone: vec4<f32>,
    storm: vec4<f32>,
    spot: vec4<f32>,
    rings: vec4<f32>,
    map: vec4<f32>,
    glow: vec4<f32>,
    sun: vec4<f32>,
    cloud: vec4<f32>,
    fixed_x: vec4<f32>,
    fixed_y: vec4<f32>,
    fixed_z: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> body: Body;
@group(constants::MATERIAL_BIND_GROUP) @binding(10) var<uniform> place: Place;

const PI: f32 = 3.14159265;
const TAU: f32 = 6.2831853;
const LIMB: f32 = 0.55;
const RINGLETS: f32 = 5.0;
const RING_DEPTH: f32 = 0.6;
const RING_EDGE: f32 = 0.04;
const ROUGHNESS: f32 = 0.95;
const F0: f32 = 0.0004;
const SHELL: f32 = 1.5;
const WIND: vec3<f32> = vec3<f32>(0.011, 0.0, 0.0045);
const CLOUD_NIGHT: f32 = 0.04;
const CLOUD_EDGE: f32 = 0.12;
const CLOUD_GRAZE: f32 = 0.08;
const CLOUD_LIMB: f32 = 0.3;
const IGN: vec3<f32> = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
const IGN_SHIFT: vec2<f32> = vec2<f32>(17.0, 31.0);

fn dither(texel: vec2<f32>) -> f32 {
    let p = texel + IGN_SHIFT;
    return fract(IGN.z * fract(p.x * IGN.x + p.y * IGN.y));
}

fn hash31(p: vec3<f32>) -> f32 {
    let q = bitcast<vec3<u32>>(vec3<i32>(p));
    var h = q.x * 0x27d4eb2du + q.y * 0x165667b1u + q.z * 0x9e3779b1u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn space_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (vec3<f32>(3.0) - 2.0 * f);
    let low = mix(
        mix(hash31(i), hash31(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
        mix(hash31(i + vec3<f32>(0.0, 1.0, 0.0)), hash31(i + vec3<f32>(1.0, 1.0, 0.0)), u.x),
        u.y
    );
    let high = mix(
        mix(hash31(i + vec3<f32>(0.0, 0.0, 1.0)), hash31(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
        mix(hash31(i + vec3<f32>(0.0, 1.0, 1.0)), hash31(i + vec3<f32>(1.0, 1.0, 1.0)), u.x),
        u.y
    );
    return mix(low, high, u.z);
}

fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var total = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var i = 0; i < octaves; i = i + 1) {
        total = total + amplitude * space_noise(q);
        q = q * 2.03 + vec3<f32>(17.1, 9.2, 4.7);
        amplitude = amplitude * 0.5;
    }
    return total;
}

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

fn scatter(f90: f32, u: f32) -> f32 {
    return 1.0 + (f90 - 1.0) * pow(1.0 - u, 5.0);
}

fn sunlit(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> f32 {
    let nl = saturate(dot(n, l));
    let nv = max(dot(n, v), 0.0001);
    let lh = saturate(dot(l, normalize(l + v)));
    let f90 = 0.5 + 2.0 * ROUGHNESS * ROUGHNESS * lh * lh;
    return scatter(f90, nl) * scatter(f90, nv) * nl / PI;
}

fn clouds(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, texel: vec2<f32>) -> vec3<f32> {
    let to_fixed = mat3x3<f32>(body.fixed_x.xyz, body.fixed_y.xyz, body.fixed_z.xyz);
    let spot = to_fixed * n * body.map.w + WIND * body.cloud.w;
    let here = fbm(spot, 4);
    let toward_light = fbm(spot + to_fixed * l * 0.3, 2);
    let facing = dot(n, v);
    let threshold = mix(0.72, 0.4, body.sun.w);
    let limb = CLOUD_LIMB * pow(1.0 - abs(facing), 4.0);
    let density = smoothstep(threshold, threshold + CLOUD_EDGE, here + limb);
    let thick = 1.0 - pow(1.0 - density, 1.0 / max(abs(facing), CLOUD_GRAZE));
    let shown = step(0.0001, body.sun.w) * body.map.z;
    if (thick * shown <= dither(texel)) {
        discard;
    }
    let relief = clamp(0.6 + (here - toward_light * 1.25) * 8.0, 0.0, 1.0);
    let day = smoothstep(-0.2, 0.25, dot(n, l));
    let base = body.cloud.rgb * mix(CLOUD_NIGHT, 1.0, day);
    return mix(base * 0.55, base * 1.05, relief) * mix(0.82, 1.05, density);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position - in.world_position.xyz);
    let facing = max(dot(n, v), 0.0);
    if (body.color.w > 0.0) {
        return vec4<f32>(body.color.rgb * body.color.w * (0.65 + 0.35 * facing), 1.0);
    }
    let l = body.light.xyz;
    if (in.uv.x > SHELL) {
        return vec4<f32>(clouds(n, v, l, in.position.xy), 1.0);
    }
    let albedo = mix(surface(n), in.color.rgb, body.map.z);
    let ember = mix(body.ember.rgb, body.glow.rgb * in.color.a, body.map.z);
    let limb = select(1.0, mix(LIMB, 1.0, sqrt(facing)), body.axis.w > 0.0);
    let here = light_at(place, dot(n, l), 0.0);
    let direct = here.sun * (sunlit(n, v, l) * ring_shade(n, l) * body.light.w * place.air.z);
    let around = ambient_response(albedo, vec3<f32>(F0), ROUGHNESS, max(facing, 0.0001)) * here.sky * place.air.z;
    let lit = (albedo * direct + around) * limb * view.exposure + ember;
    return vec4<f32>(lit, 1.0);
}
