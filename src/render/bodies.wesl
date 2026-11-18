import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Body {
    color: vec4<f32>,
    light: vec4<f32>,
    ember: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> body: Body;

const NIGHT_SIDE: f32 = 0.03;
const SHADES: f32 = 4.0;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position - in.world_position.xyz);
    let facing = max(dot(n, v), 0.0);
    let glow = body.color.rgb * body.color.w * (0.65 + 0.35 * facing);
    let day = smoothstep(-0.08, 0.35, dot(n, body.light.xyz));
    let shade = floor(day * SHADES + 0.5) / SHADES;
    let lit = body.color.rgb * mix(NIGHT_SIDE, 1.0, shade) + body.ember.rgb;
    return vec4<f32>(mix(lit, glow, step(0.001, body.color.w)), 1.0);
}
