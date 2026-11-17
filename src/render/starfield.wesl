import bevy_pbr::render::forward_io::VertexOutput;

struct Stars {
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> stars: Stars;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var tint = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    var uv = vec2<f32>(0.5, 0.5);
    @if(VERTEX_COLORS)
    tint = in.color;
    @if(VERTEX_UVS_A)
    uv = in.uv;
    let q = uv * 2.0 - vec2<f32>(1.0, 1.0);
    let core = clamp(1.0 - dot(q, q), 0.0, 1.0);
    let shape = pow(core, tint.a);
    return vec4<f32>(tint.rgb * (shape * stars.params.x), 0.0);
}
