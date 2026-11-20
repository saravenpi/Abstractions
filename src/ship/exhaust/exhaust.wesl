import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::{view, globals};

struct Jet {
    drive: vec4<f32>,
    hot: vec4<f32>,
    cool: vec4<f32>,
    form: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> jet: Jet;

const TAU: f32 = 6.2831853;
const CELLS: f32 = 7.0;
const WARP_HOT: vec3<f32> = vec3<f32>(3.5, 2.2, 7.5);
const WARP_COOL: vec3<f32> = vec3<f32>(1.2, 0.7, 3.2);

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn wrapped(x: f32) -> f32 {
    return x - CELLS * floor(x / CELLS);
}

fn noise(turn: f32, along: f32) -> f32 {
    let p = vec2<f32>(turn * CELLS, along);
    let cell = floor(p);
    let inner = fract(p);
    let blend = inner * inner * (3.0 - 2.0 * inner);
    let a = hash(vec2<f32>(wrapped(cell.x), cell.y));
    let b = hash(vec2<f32>(wrapped(cell.x + 1.0), cell.y));
    let c = hash(vec2<f32>(wrapped(cell.x), cell.y + 1.0));
    let d = hash(vec2<f32>(wrapped(cell.x + 1.0), cell.y + 1.0));
    return mix(mix(a, b, blend.x), mix(c, d, blend.x), blend.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let air = jet.drive.y;
    let warp = jet.drive.z;
    let bright = jet.hot.w;
    let scroll = jet.cool.w;
    let shock = jet.form.y;
    let layer = in.color.x;
    let turn = in.uv.x;
    let along = in.uv.y;

    let normal = normalize(in.world_normal);
    let eye = normalize(view.world_position - in.world_position.xyz);
    let face = abs(dot(normal, eye));

    let flow = along - globals.time * scroll;
    let grain = noise(turn, flow * 4.0);
    let grain_b = noise(turn + 0.31, flow * 9.0 + 3.7);
    let lines = smoothstep(0.45, 0.95, noise(turn, flow * 2.5 + 11.0));
    let fade = pow(1.0 - along, 1.3);

    let core_w = 1.0 - smoothstep(0.0, 0.4, layer);
    let mid_w = 1.0 - clamp(abs(layer - 0.5) * 3.0, 0.0, 1.0);
    let out_w = smoothstep(0.6, 1.0, layer);

    let beat = 0.5 + 0.5 * cos(along * TAU * 5.0 - 0.6);
    let diamonds = pow(beat, 6.0) * (1.0 - along) * shock;
    let kernel = core_w * (pow(face, 3.0) * fade * (0.7 + 0.5 * grain) + diamonds * 1.6);
    let mantle = mid_w * pow(face, 1.6) * fade * (0.55 + 0.9 * grain_b);
    let halo = out_w * pow(face, 0.9) * pow(1.0 - along, 0.8) * (0.25 + 0.5 * grain) * (1.5 - air);
    let streaks = warp * (core_w + mid_w) * lines * 1.5;

    let hot_rgb = mix(jet.hot.rgb, WARP_HOT, warp);
    let cool_rgb = mix(jet.cool.rgb, WARP_COOL, warp);
    let colour = hot_rgb * (kernel + streaks) + cool_rgb * (mantle + halo);
    return vec4<f32>(colour * bright, 0.0);
}
