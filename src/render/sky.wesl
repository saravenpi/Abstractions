import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Sky {
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    sun: vec4<f32>,
    moon: vec4<f32>,
    cloud: vec4<f32>,
    up: vec4<f32>,
    rim: vec4<f32>,
    disc: vec4<f32>,
    core: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: Sky;

const MOON_RADIUS: f32 = 0.045;
const RIM_SHARPNESS: f32 = 26.0;
const BAND_COLOR: vec3<f32> = vec3<f32>(0.5, 0.56, 0.78);
const BAND_GAIN: f32 = 0.07;

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q = q + dot(q, q + vec2<f32>(45.32, 45.32));
    return fract(q.x * q.y);
}

fn hash31(p: vec3<f32>) -> f32 {
    let q = bitcast<vec3<u32>>(vec3<i32>(p));
    var h = q.x * 0x27d4eb2du + q.y * 0x165667b1u + q.z * 0x9e3779b1u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0, 3.0) - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
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

fn galaxy_band(v: vec3<f32>) -> vec3<f32> {
    let side = dot(v, sky.disc.xyz);
    let toward = 0.5 + 0.5 * dot(v, sky.core.xyz);
    let width = mix(0.09, 0.24, toward * toward);
    let band = exp(-side * side / (width * width));
    let dust = space_noise(v * 7.0) * 0.6 + space_noise(v * 19.0) * 0.4;
    let lanes = smoothstep(0.22, 0.7, dust);
    return BAND_COLOR * (band * lanes * (0.35 + 0.65 * toward) * BAND_GAIN * sky.disc.w);
}

fn moon_disc(v: vec3<f32>, m: vec3<f32>) -> vec4<f32> {
    let t = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), m));
    let b = cross(m, t);
    let q = vec2<f32>(dot(v, t), dot(v, b)) / MOON_RADIUS;
    let r2 = dot(q, q);
    if (dot(v, m) <= 0.0 || r2 >= 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let n = vec3<f32>(q, sqrt(1.0 - r2));
    let blotch = value_noise(q * 2.2 + vec2<f32>(5.0, 3.0)) * 0.65 + value_noise(q * 5.0) * 0.35;
    let maria = smoothstep(0.42, 0.62, blotch);
    let albedo = mix(vec3<f32>(0.92, 0.94, 1.0), vec3<f32>(0.5, 0.56, 0.68), maria * 0.8);
    let lit = smoothstep(-0.1, 0.3, dot(n, normalize(vec3<f32>(0.5, 0.2, 0.85))));
    let edge = 1.0 - smoothstep(0.9, 1.0, r2);
    return vec4<f32>(albedo * mix(0.1, 1.0, lit), edge);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = normalize(in.world_position.xyz - view.world_position);
    let sun_dir = sky.sun.xyz;
    let moon_dir = sky.moon.xyz;
    let height = dot(v, sky.up.xyz);
    let over_limb = height + sky.up.w;
    let above_horizon = smoothstep(-0.01, 0.02, over_limb);

    var col = mix(sky.horizon.rgb, sky.zenith.rgb, pow(clamp(height, 0.0, 1.0), 0.5));
    col = col + sky.rim.rgb * exp(-abs(over_limb) * RIM_SHARPNESS);
    col = col + galaxy_band(v) * above_horizon;

    let sun_visible = sky.sun.w * above_horizon;
    let sun_col = mix(vec3<f32>(1.0, 0.5, 0.2), vec3<f32>(1.0, 0.95, 0.8), smoothstep(0.0, 0.4, dot(sun_dir, sky.up.xyz)));
    let sun_dist = 1.0 - dot(v, sun_dir);
    let sun_glow = exp(-sun_dist * 45.0) * 0.55 + exp(-sun_dist * 6.0) * 0.18;
    col = col + sun_col * sun_glow * sun_visible;

    let moon_visible = sky.moon.w * above_horizon;
    let moon_dist = 1.0 - dot(v, moon_dir);
    col = col + vec3<f32>(0.3, 0.4, 0.65) * exp(-moon_dist * 70.0) * 0.35 * moon_visible;
    let moon = moon_disc(v, moon_dir);
    col = mix(col, moon.rgb * 1.6, moon.a * moon_visible);

    col = mix(col, sky.cloud.rgb, sky.cloud.w);
    return vec4<f32>(col, 1.0);
}
