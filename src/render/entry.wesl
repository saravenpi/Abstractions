import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Fx {
    heading: vec4<f32>,
    core: vec4<f32>,
    wisp: vec4<f32>,
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> fx: Fx;

const TAU: f32 = 6.2831853;
const WISP_LANES: f32 = 22.0;
const CORE_TIGHT: f32 = 60.0;
const CORE_WIDE: f32 = 5.0;
const FLASH_WIDE: f32 = 1.5;

fn hash11(x: f32) -> f32 {
    var h = bitcast<u32>(i32(x)) * 0x27d4eb2du;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn lanes(turn: f32, along: f32, count: f32, width: f32, travel: f32) -> f32 {
    let scaled = turn * count;
    let lane = floor(scaled);
    let seed = hash11(lane);
    let across = abs(scaled - lane - 0.5);
    let line = 1.0 - smoothstep(width * 0.5, width, across);
    let dash = fract(along * (1.5 + seed) + travel * (0.6 + seed) + seed * 9.0);
    let lit = smoothstep(0.0, 0.15, dash) * (1.0 - smoothstep(0.35, 0.6, dash));
    return line * lit * step(0.45, seed);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = normalize(in.world_position.xyz - view.world_position);
    let axis = fx.heading.xyz;
    let time = fx.params.x;

    let along = dot(v, axis);
    let radial = v - axis * along;
    let spread = length(radial);
    let pick = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), step(0.9, abs(axis.y)));
    let side = normalize(cross(axis, pick));
    let rise = cross(axis, side);
    let turn = atan2(dot(radial, rise), dot(radial, side)) / TAU + 0.5;
    let open = smoothstep(0.04, 0.3, spread);

    let wisps = lanes(turn, along, WISP_LANES, 0.5, time * fx.params.z) * open;

    let off = 1.0 - along;
    let ahead = 0.6 * exp(-off * CORE_TIGHT) + 0.4 * exp(-off * CORE_WIDE);
    let flash = fx.params.y * exp(-off * FLASH_WIDE);

    var col = fx.core.rgb * (fx.core.w * ahead + flash);
    col = col + fx.wisp.rgb * (fx.wisp.w * wisps);
    return vec4<f32>(col, 0.0);
}
