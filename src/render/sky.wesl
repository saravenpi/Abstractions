import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Sky {
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    sun: vec4<f32>,
    cloud: vec4<f32>,
    up: vec4<f32>,
    rim: vec4<f32>,
    galaxy: vec4<f32>,
    to_galaxy_x: vec4<f32>,
    to_galaxy_y: vec4<f32>,
    to_galaxy_z: vec4<f32>,
    glow: vec4<f32>,
    warp: vec4<f32>,
}

struct SkyOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: Sky;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var galaxy_map: texture_cube<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var galaxy_sampler: sampler;

const RIM_SHARPNESS: f32 = 26.0;
const GALAXY_GAIN: f32 = 0.07;
const LEAST_DARK: f32 = 0.001;
const BEAM: f32 = 1.5;
const BEAM_LEAST: f32 = 0.08;
const BEAM_MOST: f32 = 3.0;

fn galaxy_light(v: vec3<f32>) -> vec3<f32> {
    let to_galaxy = mat3x3<f32>(sky.to_galaxy_x.xyz, sky.to_galaxy_y.xyz, sky.to_galaxy_z.xyz);
    let held = textureSampleLevel(galaxy_map, galaxy_sampler, to_galaxy * v, 0.0).rgb;
    let seen = -log(max(vec3<f32>(1.0) - held, vec3<f32>(LEAST_DARK)));
    return seen * (GALAXY_GAIN * sky.galaxy.x * sky.galaxy.y);
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
    return galaxy_light(dir) * clamp(pow(factor, BEAM), BEAM_LEAST, BEAM_MOST);
}

@fragment
fn fragment(in: VertexOutput) -> SkyOut {
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
    return SkyOut(vec4<f32>(col, 1.0), 0.0);
}
