import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::{view, fog};
import package::render::cloud_field::{Puff, broad_cover, day_side, density, dither, fbm_seen, lone, puff, spot_at};

struct Clouds {
    light: vec4<f32>,
    color: vec4<f32>,
    haze: vec4<f32>,
    params: vec4<f32>,
    space: vec4<f32>,
    ambient: vec4<f32>,
    drift: vec4<f32>,
    fixed_x: vec4<f32>,
    fixed_y: vec4<f32>,
    fixed_z: vec4<f32>,
    fade: vec4<f32>,
    eye: vec4<f32>,
    slice: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> clouds: Clouds;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var cover_map: texture_cube<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var cover_sampler: sampler;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const EDGE_PIXELS: f32 = 1.5;
const SHADE_SKY: f32 = 0.45;
const SHADE_LEVEL: f32 = 0.62;

fn strongest(v: vec3<f32>) -> vec3<f32> {
    return v / max(max(v.x, max(v.y, v.z)), 0.001);
}

fn shaded(base: vec3<f32>, cloud: Puff, above: bool, ambient: vec3<f32>) -> vec3<f32> {
    let body = select(mix(1.0, 0.72, cloud.density), mix(0.82, 1.05, cloud.density), above);
    let sky = mix(vec3<f32>(1.0), strongest(ambient), SHADE_SKY);
    let dark = sky * (dot(base, LUMA) * SHADE_LEVEL / max(dot(sky, LUMA), 0.01));
    return mix(dark, base * 1.05, cloud.relief) * body;
}

fn around(ray: vec3<f32>, to_fixed: mat3x3<f32>, cover: f32) -> f32 {
    let step = to_fixed * ray / clouds.eye.w;
    var dense = 0.0;
    for (var k = 1; k <= 4; k = k + 1) {
        let metres = 40.0 * f32(k * k);
        let at = spot_at(clouds.eye.xyz + step * metres, clouds.drift);
        dense = dense + density(fbm_seen(at, 4, 0.0), cover);
    }
    return dense * 0.25;
}

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
    let spot = dir * clouds.params.x;
    let to_eye = view.world_position - in.world_position.xyz;
    let dist = length(to_eye);
    let facing = dot(n, to_eye / max(dist, 0.001));
    let footprint = max(length(dpdx(spot)), length(dpdy(spot)));
    let cloud = puff(spot, clouds.drift, to_fixed * light, facing, cover, footprint, i32(clouds.slice.y));
    let through = exp(-clouds.fade.xyz * dist);
    let body = cloud.density * lone(cloud, cover) - 0.5 * clouds.slice.x;
    let edge = clamp(0.5 + (body - 0.5) / max(fwidth(body) * EDGE_PIXELS, 0.0001), 0.0, 1.0);
    let alpha = max(edge, whiteout);
    if (alpha <= dither(in.position.xy)) {
        discard;
    }

    let base = mix(day_side(clouds.space.rgb, n, light), clouds.color.rgb, air);
    var col = shaded(base, cloud, facing > 0.0, clouds.ambient.rgb);
    col = col * through + fog.base_color.rgb * (vec3<f32>(1.0) - through);
    if (whiteout > 0.0) {
        let near = around(-to_eye / max(dist, 0.001), to_fixed, cover);
        let veil = whiteout * mix(0.55, 1.0, near);
        col = mix(col, clouds.color.rgb * mix(0.8, 1.04, near), veil);
    }
    return vec4<f32>(col, 1.0);
}
