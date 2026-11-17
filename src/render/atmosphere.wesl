import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Halo {
    color: vec4<f32>,
    light: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> halo: Halo;

const HAZE: f32 = 0.5;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position - in.world_position.xyz);
    let facing = max(dot(n, v), 0.0);
    let limb = halo.light.w;
    let outside = smoothstep(0.0, limb, facing);
    let inside = 1.0 - smoothstep(limb, 1.0, facing);
    let glow = mix(inside * inside * inside * HAZE, outside * outside, step(facing, limb));
    let day = smoothstep(-0.3, 0.3, dot(n, halo.light.xyz));
    return vec4<f32>(halo.color.rgb * (halo.color.w * glow * day), 0.0);
}
