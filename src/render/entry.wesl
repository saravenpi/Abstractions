import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Fx {
    heading: vec4<f32>,
    core: vec4<f32>,
    flash: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> fx: Fx;

const CORE_TIGHT: f32 = 60.0;
const CORE_WIDE: f32 = 5.0;
const FLASH_WIDE: f32 = 1.5;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = normalize(in.world_position.xyz - view.world_position);
    let off = 1.0 - dot(v, fx.heading.xyz);
    let ahead = 0.6 * exp(-off * CORE_TIGHT) + 0.4 * exp(-off * CORE_WIDE);
    let flash = fx.flash.x * exp(-off * FLASH_WIDE);
    return vec4<f32>(fx.core.rgb * (fx.core.w * ahead + flash), 0.0);
}
