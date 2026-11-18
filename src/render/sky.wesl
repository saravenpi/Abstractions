import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Sky {
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    sun: vec4<f32>,
    cloud: vec4<f32>,
    up: vec4<f32>,
    rim: vec4<f32>,
    disc: vec4<f32>,
    core: vec4<f32>,
    glow: vec4<f32>,
    warp: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: Sky;

const RIM_SHARPNESS: f32 = 26.0;
const BAND_COLOR: vec3<f32> = vec3<f32>(0.5, 0.56, 0.78);
const BAND_GAIN: f32 = 0.07;
const BEAM: f32 = 1.5;
const BEAM_LEAST: f32 = 0.08;
const BEAM_MOST: f32 = 3.0;

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

fn galaxy_band(v: vec3<f32>) -> vec3<f32> {
    let side = dot(v, sky.disc.xyz);
    let toward = 0.5 + 0.5 * dot(v, sky.core.xyz);
    let width = mix(0.09, 0.24, toward * toward);
    let band = exp(-side * side / (width * width));
    let dust = space_noise(v * 7.0) * 0.6 + space_noise(v * 19.0) * 0.4;
    let lanes = smoothstep(0.22, 0.7, dust);
    return BAND_COLOR * (band * lanes * (0.35 + 0.65 * toward) * BAND_GAIN * sky.disc.w);
}

fn warped_band(v: vec3<f32>) -> vec3<f32> {
    let axis = sky.warp.xyz;
    let beta = sky.warp.w;
    let cos_seen = dot(v, axis);
    let lorentz = sqrt(max(1.0 - beta * beta, 0.000001));
    let shrink = 1.0 - beta * cos_seen;
    let bent = axis * ((cos_seen - beta) / shrink) + (v - axis * cos_seen) * (lorentz / shrink);
    let moving = beta > 0.0;
    let dir = select(v, normalize(bent), moving);
    let factor = select(1.0, lorentz / shrink, moving);
    return galaxy_band(dir) * clamp(pow(factor, BEAM), BEAM_LEAST, BEAM_MOST);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = normalize(in.world_position.xyz - view.world_position);
    let sun_dir = sky.sun.xyz;
    let height = dot(v, sky.up.xyz);
    let over_limb = height + sky.up.w;
    let above_horizon = smoothstep(-0.01, 0.02, over_limb);

    var col = mix(sky.horizon.rgb, sky.zenith.rgb, pow(clamp(height, 0.0, 1.0), 0.5));
    col = col + sky.rim.rgb * exp(-abs(over_limb) * RIM_SHARPNESS);
    col = col + warped_band(v) * above_horizon;

    let sun_visible = sky.sun.w * above_horizon;
    let sun_col = sky.glow.rgb;
    let sun_dist = 1.0 - dot(v, sun_dir);
    let sun_glow = exp(-sun_dist * 45.0) * 0.55 + exp(-sun_dist * 6.0) * 0.18;
    col = col + sun_col * sun_glow * sun_visible;

    col = mix(col, sky.cloud.rgb, sky.cloud.w);
    return vec4<f32>(col, 1.0);
}
