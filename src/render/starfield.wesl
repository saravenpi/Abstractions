import bevy_pbr::render::forward_io::{Vertex, decompress_vertex};
import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::view_transformations::position_world_to_clip;

struct Stars {
    params: vec4<f32>,
    heading: vec4<f32>,
    drift: vec4<f32>,
}

struct StarVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) spot: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) half: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> stars: Stars;

const EXTENDED: f32 = 50.0;
const GALAXY_ANGLE: f32 = 0.35;
const LOG10: f32 = 2.302585;
const BEAM: f32 = 1.5;
const BEAM_LEAST: f32 = 0.08;
const BEAM_MOST: f32 = 3.0;
const SHIFT_SPAN: f32 = 2.0;
const BLUE: vec3<f32> = vec3<f32>(0.8, 0.92, 1.25);
const RED: vec3<f32> = vec3<f32>(1.2, 0.8, 0.55);
const SPREAD: f32 = 0.6;
const FOUR_OVER_PI: f32 = 1.2732395;

fn shine(flux: f32) -> vec2<f32> {
    let bright = clamp((flux + 4.6) / 5.0, 0.08, 1.0);
    return vec2<f32>(0.0035 + 0.0055 * bright, 0.45 + 2.6 * bright * bright);
}

fn tint(factor: f32) -> vec3<f32> {
    let shift = clamp(log2(max(factor, 0.000001)) / SHIFT_SPAN, -1.0, 1.0);
    let toward = mix(RED, BLUE, step(0.0, shift));
    return mix(vec3<f32>(1.0), toward, abs(shift));
}

@vertex
fn vertex(vertex_in: Vertex) -> StarVertex {
    var out: StarVertex;
    let vertex = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    let corner = vertex.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let built = -vertex.normal;
    let range = max(vertex.uv_b.x, 0.0001);
    let extended = step(EXTENDED, vertex.uv_b.y);

    let far = built * range - stars.drift.xyz;
    let now = max(length(far), 0.0001);
    let was = shine(vertex.uv_b.y);
    let lit = shine(vertex.uv_b.y - 2.0 * log(now / range) / LOG10);
    let drawn = length(vertex.position - built) * 0.70710678;
    let star = vec2<f32>(lit.x, lit.y / was.y);
    let galaxy = vec2<f32>(min(drawn * range / now, GALAXY_ANGLE), 1.0);
    let sized = mix(star, galaxy, extended);

    let shell = length(world_from_local[0].xyz);
    let dir = normalize((world_from_local * vec4<f32>(far / now, 0.0)).xyz);
    let axis = stars.heading.xyz;
    let beta = stars.heading.w;
    let cos_true = dot(dir, axis);
    let radial = dir - axis * cos_true;
    let sin_true = length(radial);
    let pick = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), step(0.9, abs(axis.y)));
    let aside = normalize(cross(axis, pick));
    let outward = select(aside, radial / max(sin_true, 0.000001), sin_true > 0.0001);
    let lorentz = sqrt(max(1.0 - beta * beta, 0.000001));
    let cos_seen = clamp((cos_true + beta) / (1.0 + beta * cos_true), -1.0, 1.0);
    let sin_seen = sin_true * lorentz / (1.0 + beta * cos_true);
    let seen = axis * cos_seen + outward * sin_seen;
    let along = outward * cos_seen - axis * sin_seen;
    let across = cross(seen, along);

    let factor = (1.0 + beta * cos_true) / lorentz;
    let size = sized.x * mix(1.0, 1.0 / factor, extended);
    let reach = 0.5 * stars.params.y * sin_seen;
    let half = reach / size;
    let place = seen + across * (corner.x * size) + along * (corner.y * (size + reach));
    let beam = clamp(pow(factor, BEAM), BEAM_LEAST, BEAM_MOST);
    let energy = pow(1.0 + half * FOUR_OVER_PI, -SPREAD);
    let share = stars.params.x * mix(stars.params.z, 1.0, extended);
    let light = sized.y * beam * energy * share;

    out.position = position_world_to_clip(world_from_local[3].xyz + place * shell);
    out.spot = vec2<f32>(corner.x, corner.y * (1.0 + half));
    out.color = vec4<f32>(vertex.color.rgb * tint(factor) * light, vertex.color.a);
    out.half = half;
    return out;
}

@fragment
fn fragment(in: StarVertex) -> @location(0) vec4<f32> {
    let q = vec2<f32>(in.spot.x, max(abs(in.spot.y) - in.half, 0.0));
    let core = clamp(1.0 - dot(q, q), 0.0, 1.0);
    let shape = pow(core, in.color.a);
    return vec4<f32>(in.color.rgb * shape, 0.0);
}
