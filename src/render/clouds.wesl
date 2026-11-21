import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;
import package::render::cloud_field::{broad_cover, day_side, dither, lone, puff, shade};

struct Clouds {
    light: vec4<f32>,
    color: vec4<f32>,
    haze: vec4<f32>,
    params: vec4<f32>,
    space: vec4<f32>,
    drift: vec4<f32>,
    fixed_x: vec4<f32>,
    fixed_y: vec4<f32>,
    fixed_z: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> clouds: Clouds;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var cover_map: texture_cube<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var cover_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let to_fixed = mat3x3<f32>(clouds.fixed_x.xyz, clouds.fixed_y.xyz, clouds.fixed_z.xyz);
    let n = normalize(in.world_normal);
    let light = clouds.light.xyz;
    let air = clouds.light.w;
    let whiteout = clouds.haze.w;

    let dir = to_fixed * n;
    let mapped = broad_cover(cover_map, cover_sampler, dir, clouds.color.w);
    let cover = select(mapped, clouds.params.w, clouds.params.z > 0.5);
    let spot = dir * clouds.params.x + clouds.drift.xyz;
    let to_eye = view.world_position - in.world_position.xyz;
    let dist = length(to_eye);
    let facing = dot(n, to_eye / max(dist, 0.001));
    let cloud = puff(spot, to_fixed * light, facing, cover);
    let far = smoothstep(0.35, 1.0, dist / max(clouds.params.y, 1.0));
    let alpha = max(cloud.thick * lone(cloud), whiteout);
    if (alpha <= dither(in.position.xy)) {
        discard;
    }

    let base = mix(day_side(clouds.space.rgb, n, light), clouds.color.rgb, air);
    var col = shade(base, cloud, facing > 0.0);
    col = mix(col, clouds.haze.rgb, far * air);
    col = mix(col, clouds.color.rgb, whiteout);
    return vec4<f32>(col, 1.0);
}
