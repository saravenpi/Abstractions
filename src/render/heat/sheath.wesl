import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::mesh_view_bindings::globals;

struct Sheath {
    centre: vec4<f32>,
    axis: vec4<f32>,
    glow: vec4<f32>,
    load: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sheath: Sheath;

const TAU: f32 = 6.2831853;
const REACH: f32 = 2.4;
const LANES: f32 = 26.0;
const TAIL: f32 = 0.9;

fn hash11(x: f32) -> f32 {
    var h = bitcast<u32>(i32(x)) * 0x27d4eb2du;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn angle(local: vec3<f32>, axis: vec3<f32>) -> f32 {
    let pick = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), step(0.9, abs(axis.y)));
    let side = normalize(cross(axis, pick));
    let rise = cross(axis, side);
    return atan2(dot(local, rise), dot(local, side)) / TAU + 0.5;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = sheath.centre.w;
    let axis = sheath.axis.xyz;
    let local = (in.world_position.xyz - sheath.centre.xyz) / radius;
    let along = dot(local, axis);
    let across = length(local - axis * along);
    let time = globals.time;
    let heat = sheath.glow.w;

    let front = 1.1 - 0.7 * across * across;
    let shock = exp(-pow((along - front) * 3.2, 2.0)) * step(-0.3, along) * step(across, 1.5);
    let inner = smoothstep(front, front - 1.2, along) * smoothstep(1.5, 0.3, across) * step(-0.3, along);

    let turn = angle(local, axis);
    let lane = floor(turn * LANES);
    let seed = hash11(lane);
    let flow = along * 2.0 + time * (2.0 + 3.0 * seed);
    let dash = fract(flow + seed * 9.0);
    let streak = smoothstep(0.0, 0.2, dash) * (1.0 - smoothstep(0.3, 0.7, dash));
    let line = 1.0 - smoothstep(0.2, 0.5, abs(fract(turn * LANES) - 0.5) * 2.0);
    let plasma = streak * line * step(0.4, seed) * smoothstep(0.25, 1.0, across) * step(-0.4, along);

    let tail = exp(along * TAIL) * step(along, 0.0) * smoothstep(0.7, 0.0, across);
    let wake = tail * (0.6 + 0.4 * hash11(lane + floor(time * 14.0) * 31.0)) * step(0.3, seed);

    let cell = floor(along * 5.0 + time * (4.0 + 5.0 * seed)) + lane * 131.0;
    let ember = step(0.86, hash11(cell)) * (1.0 - smoothstep(0.0, 0.5, abs(fract(along * 5.0 + time * (4.0 + 5.0 * seed)) - 0.5) * 2.0)) * step(along, 0.6) * smoothstep(0.3, 0.9, across) * smoothstep(1.4, 0.9, across);

    let body = 0.9 * shock + 0.25 * inner + 0.8 * plasma + 0.5 * wake;
    let sparks = ember * (0.6 + sheath.load.x);
    let colour = sheath.glow.rgb * (heat * body) + mix(sheath.glow.rgb, vec3<f32>(1.0), 0.5) * (heat * sparks);
    return vec4<f32>(colour, 0.0);
}
