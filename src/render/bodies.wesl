import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;
import bevy_pbr::render::pbr_types::pbr_input_new;
import package::render::cloud_field::{OCTAVES, broad_cover, day_side, dither, lone, puff, shade};
import package::render::daylight::{Place, light_at, sky_response, sun_response, transmit};
import package::render::season::{Season, look_at, tinted};

struct Body {
    color: vec4<f32>,
    light: vec4<f32>,
    ember: vec4<f32>,
    axis: vec4<f32>,
    tone: vec4<f32>,
    storm: vec4<f32>,
    spot: vec4<f32>,
    rings: vec4<f32>,
    map: vec4<f32>,
    glow: vec4<f32>,
    sun: vec4<f32>,
    cloud: vec4<f32>,
    fixed_x: vec4<f32>,
    fixed_y: vec4<f32>,
    fixed_z: vec4<f32>,
    drift: vec4<f32>,
    orb: vec4<f32>,
}

struct Seen {
    n: vec3<f32>,
    v: vec3<f32>,
    at: vec3<f32>,
    miss: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> body: Body;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var cover_map: texture_cube<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var cover_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var surface_map: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var surface_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(5) var relief_map: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(6) var relief_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(10) var<uniform> place: Place;
@group(constants::MATERIAL_BIND_GROUP) @binding(11) var<uniform> season: Season;

const TAU: f32 = 6.2831853;
const LIMB: f32 = 0.55;
const RINGLETS: f32 = 5.0;
const RING_DEPTH: f32 = 0.6;
const RING_EDGE: f32 = 0.04;
const ROUGHNESS: f32 = 0.95;
const REFLECTANCE: f32 = 0.05;
const SHELL: f32 = 1.5;
const QUARTER: f32 = 0.78539816;
const SCATTER: f32 = 0.0795775;

fn ring_cover(u: f32, inner: f32, density: f32) -> f32 {
    if (u < inner || u > 1.0) {
        return 0.0;
    }
    let t = (u - inner) / max(1.0 - inner, 0.0001);
    let edge = smoothstep(inner, inner + RING_EDGE, u) * (1.0 - smoothstep(1.0 - RING_EDGE, 1.0, u));
    let wave = TAU * RINGLETS * t + 1.7 * sin(TAU * 1.3 * t);
    return density * edge * (1.0 - RING_DEPTH * (0.5 + 0.5 * cos(wave)));
}

fn ring_shade(n: vec3<f32>, l: vec3<f32>) -> f32 {
    let outer = body.rings.y;
    let axis = body.axis.xyz;
    let height = dot(n, axis);
    let rise = dot(l, axis);
    if (outer <= 0.0 || height * rise >= 0.0) {
        return 1.0;
    }
    let hit = n - l * (height / rise);
    return 1.0 - ring_cover(length(hit) / outer, body.rings.x / outer, body.rings.z);
}

fn band(lat: f32) -> f32 {
    let k = body.axis.w;
    let wave = sin(lat * k + body.tone.w + 0.8 * sin(lat * k * 0.43 + 1.3));
    return smoothstep(-0.35, 0.35, wave);
}

fn surface(n: vec3<f32>) -> vec3<f32> {
    if (body.axis.w <= 0.0) {
        return body.color.rgb;
    }
    let lat = asin(clamp(dot(n, body.axis.xyz), -1.0, 1.0));
    let banded = mix(body.color.rgb, body.tone.rgb, band(lat));
    let size = body.storm.w;
    let spot = smoothstep(size, mix(size, 1.0, 0.5), dot(n, body.storm.xyz));
    return mix(banded, body.spot.rgb, spot);
}

fn clouds(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, texel: vec2<f32>, span: f32) -> vec3<f32> {
    let to_fixed = mat3x3<f32>(body.fixed_x.xyz, body.fixed_y.xyz, body.fixed_z.xyz);
    let dir = to_fixed * n;
    let cover = broad_cover(cover_map, cover_sampler, dir, body.cloud.w);
    let spot = dir * body.map.w;
    let cloud = puff(spot, body.drift, to_fixed * l, dot(n, v), cover, span * body.map.w, OCTAVES);
    if (cloud.density * lone(cloud, cover) * body.sun.w * body.map.z <= dither(texel)) {
        discard;
    }
    return shade(day_side(body.cloud.rgb, n, l), cloud, true);
}

fn face_of(d: vec3<f32>) -> vec3<f32> {
    let a = abs(d);
    var layer = 0.0;
    var n = vec3<f32>(1.0, 0.0, 0.0);
    var u = vec3<f32>(0.0, 0.0, 1.0);
    var v = vec3<f32>(0.0, 1.0, 0.0);
    if (a.x >= a.y && a.x >= a.z) {
        if (d.x <= 0.0) {
            layer = 1.0;
            n = vec3<f32>(-1.0, 0.0, 0.0);
            u = vec3<f32>(0.0, 1.0, 0.0);
            v = vec3<f32>(0.0, 0.0, 1.0);
        }
    } else if (a.y >= a.z) {
        layer = select(3.0, 2.0, d.y > 0.0);
        n = vec3<f32>(0.0, sign(d.y), 0.0);
        u = vec3<f32>(1.0, 0.0, 0.0);
        v = vec3<f32>(0.0, 0.0, sign(d.y));
    } else {
        layer = select(5.0, 4.0, d.z > 0.0);
        n = vec3<f32>(0.0, 0.0, sign(d.z));
        u = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), d.z > 0.0);
        v = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), d.z > 0.0);
    }
    let depth = dot(d, n);
    let at = vec2<f32>(atan(dot(d, u) / depth), atan(dot(d, v) / depth)) / QUARTER;
    return vec3<f32>(at * 0.5 + 0.5, layer);
}

fn face_at(n: vec3<f32>, span: f32) -> vec4<f32> {
    let to_fixed = mat3x3<f32>(body.fixed_x.xyz, body.fixed_y.xyz, body.fixed_z.xyz);
    let at = face_of(to_fixed * n);
    let side = f32(textureDimensions(surface_map).x);
    let lod = max(log2(max(span, 1.0e-6) * side / (2.0 * QUARTER)), 0.0);
    return vec4<f32>(at, lod);
}

fn lean(at: vec4<f32>) -> vec3<f32> {
    let stored = textureSampleLevel(relief_map, relief_sampler, at.xy, i32(at.z), at.w).xyz;
    let to_fixed = mat3x3<f32>(body.fixed_x.xyz, body.fixed_y.xyz, body.fixed_z.xyz);
    return transpose(to_fixed) * (stored * 2.0 - 1.0);
}

fn seen(in: VertexOutput) -> Seen {
    let eye = view.world_position;
    var out: Seen;
    out.n = normalize(in.world_normal);
    out.v = normalize(eye - in.world_position.xyz);
    out.at = in.world_position.xyz;
    out.miss = 0.0;
    let radius = select(body.orb.w, body.map.x, in.uv.x > SHELL);
    if (radius <= 0.0) {
        return out;
    }
    let d = -out.v;
    let o = (eye - body.orb.xyz) / radius;
    let p = o - d * dot(o, d);
    let off = dot(p, p);
    out.n = normalize(p - d * sqrt(max(1.0 - off, 0.0)));
    out.at = body.orb.xyz + out.n * radius;
    out.miss = off - 1.0;
    return out;
}

fn veiled(lit: vec3<f32>, sun: vec3<f32>, facing: f32) -> vec3<f32> {
    let share = place.colour.w;
    if (share <= 0.0) {
        return lit;
    }
    let through = mix(vec3<f32>(1.0), transmit(facing, place.star.w), share);
    let haze = sun * (SCATTER * place.air.z) * view.exposure;
    return lit * through + haze * (vec3<f32>(1.0) - through);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let hit = seen(in);
    let n = hit.n;
    let v = hit.v;
    let facing = max(dot(n, v), 0.0);
    let span = max(length(dpdx(n)), length(dpdy(n)));
    if (hit.miss > 0.0) {
        discard;
    }
    if (body.color.w > 0.0) {
        return vec4<f32>(body.color.rgb * body.color.w * (0.65 + 0.35 * facing), 1.0);
    }
    let l = body.light.xyz;
    if (in.uv.x > SHELL) {
        return vec4<f32>(clouds(n, v, l, in.position.xy, span), 1.0);
    }
    var mapped = vec4<f32>(body.color.rgb, 0.0);
    var tilt = vec3<f32>(0.0);
    if (body.glow.w > 0.0) {
        let at = face_at(n, span);
        mapped = textureSampleLevel(surface_map, surface_sampler, at.xy, i32(at.z), at.w);
        tilt = lean(at);
    }
    let baked = tinted(mapped.rgb, look_at(season, n, 0.0));
    let albedo = mix(surface(n), baked, body.map.z);
    let ember = mix(body.ember.rgb, body.glow.rgb * mapped.a, body.map.z);
    let limb = select(1.0, mix(LIMB, 1.0, sqrt(facing)), body.axis.w > 0.0);
    let bent = normalize(n + tilt * body.map.z);
    var ground = pbr_input_new();
    ground.material.base_color = vec4<f32>(albedo, 1.0);
    ground.material.perceptual_roughness = ROUGHNESS;
    ground.material.reflectance = vec3<f32>(REFLECTANCE);
    ground.world_position = vec4<f32>(hit.at, 1.0);
    ground.world_normal = bent;
    ground.N = bent;
    ground.V = v;
    let here = light_at(place, dot(n, l), 0.0);
    let direct = here.sun * (sun_response(ground, l) * (ring_shade(n, l) * body.light.w * place.air.z));
    let around = sky_response(ground) * here.sky * place.air.z;
    let lit = (direct + around) * limb * view.exposure + ember;
    return vec4<f32>(veiled(lit, here.sun, facing), 1.0);
}
