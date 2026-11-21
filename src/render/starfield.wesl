import bevy_pbr::render::forward_io::{UncompressedVertex, Vertex, decompress_vertex};
import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::view_transformations::position_world_to_clip;

struct Stars {
    params: vec4<f32>,
    heading: vec4<f32>,
    drift: vec4<f32>,
    flow: vec4<f32>,
    dome: vec4<f32>,
    base: vec4<f32>,
}

struct StarVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) spot: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) half: f32,
}

struct Seen {
    dir: vec3<f32>,
    along: vec3<f32>,
    factor: f32,
    turn: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> stars: Stars;

const LOG10: f32 = 2.302585;
const BEAM: f32 = 1.5;
const BEAM_LEAST: f32 = 0.08;
const BEAM_MOST: f32 = 3.0;
const SHIFT_SPAN: f32 = 2.0;
const BLUE: vec3<f32> = vec3<f32>(0.8, 0.92, 1.25);
const RED: vec3<f32> = vec3<f32>(1.2, 0.8, 0.55);
const FOUR_OVER_PI: f32 = 1.2732395;
const TAU: f32 = 6.2831853;
const DUST: f32 = -100.0;
const MOTES: u32 = 160u;
const DEPTH_AHEAD: f32 = 8.0;
const DEPTH_BEHIND: f32 = -2.0;
const TUBE_NEAR: f32 = 0.12;
const TUBE_FAR: f32 = 1.0;
const FADE_IN: f32 = 0.2;
const FADE_OUT: f32 = 0.85;
const EXPOSURE: f32 = 0.041666668;
const GRAIN: f32 = 0.002;
const GRAIN_ASPECT: f32 = 4.0;
const GRAIN_REACH: f32 = 0.02;
const GRAIN_LIGHT: f32 = 0.08;
const GRAIN_PACE: f32 = 0.01;
const STREAK_MOST: f32 = 0.25;
const SIGHT: f32 = 40.0;
const SIGHT_FADE: f32 = 2.5;
const DAY_FLUX: f32 = 1.0;
const SKY_DAY: f32 = 0.3;
const SKY_SPAN: f32 = 3.0;
const SKY_LEAST: f32 = 0.0001;

fn lit_by_night(flux: f32, dir: vec3<f32>) -> f32 {
    let height = clamp(dot(dir, stars.dome.xyz), 0.0, 1.0);
    let sky = mix(stars.base.x, stars.dome.w, sqrt(height));
    let hide = min(DAY_FLUX + SKY_SPAN * log(max(sky, SKY_LEAST) / SKY_DAY) / LOG10, DAY_FLUX);
    return smoothstep(hide, hide + 1.0, flux);
}

fn shine(flux: f32) -> vec2<f32> {
    let bright = clamp((flux + 4.6) / 5.0, 0.08, 1.0);
    return vec2<f32>(0.0035 + 0.0055 * bright, 0.45 + 2.6 * bright * bright);
}

fn tint(factor: f32) -> vec3<f32> {
    let shift = clamp(log2(max(factor, 0.000001)) / SHIFT_SPAN, -1.0, 1.0);
    let toward = mix(RED, BLUE, step(0.0, shift));
    return mix(vec3<f32>(1.0), toward, abs(shift));
}

fn hash(key: u32) -> f32 {
    var h = key * 0x27d4eb2du;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn aside(axis: vec3<f32>) -> vec3<f32> {
    let pick = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), step(0.9, abs(axis.y)));
    return normalize(cross(axis, pick));
}

fn see(dir: vec3<f32>) -> Seen {
    let axis = stars.heading.xyz;
    let beta = stars.heading.w;
    let cos_true = dot(dir, axis);
    let radial = dir - axis * cos_true;
    let sin_true = length(radial);
    let outward = select(aside(axis), radial / max(sin_true, 0.000001), sin_true > 0.0001);
    let lorentz = sqrt(max(1.0 - beta * beta, 0.000001));
    let cos_seen = clamp((cos_true + beta) / (1.0 + beta * cos_true), -1.0, 1.0);
    let sin_seen = sin_true * lorentz / (1.0 + beta * cos_true);
    var seen: Seen;
    seen.dir = axis * cos_seen + outward * sin_seen;
    seen.along = outward * cos_seen - axis * sin_seen;
    seen.factor = (1.0 + beta * cos_true) / lorentz;
    seen.turn = sin_true;
    return seen;
}

fn window(t: f32) -> f32 {
    let rise = smoothstep(0.0, 1.0, t / FADE_IN);
    let fall = smoothstep(0.0, 1.0, (1.0 - t) / (1.0 - FADE_OUT));
    return rise * fall;
}

fn beamed(factor: f32, half: f32) -> vec3<f32> {
    let beam = clamp(pow(factor, BEAM), BEAM_LEAST, BEAM_MOST);
    return tint(factor) * (beam / (1.0 + half * FOUR_OVER_PI));
}

fn emit(seen: Seen, corner: vec2<f32>, size: f32, reach: f32, light: vec4<f32>) -> StarVertex {
    var out: StarVertex;
    let half = reach / size;
    let across = cross(seen.dir, seen.along);
    let place = seen.dir + across * (corner.x * size) + seen.along * (corner.y * (size + reach));
    out.position = vec4<f32>(place, 1.0);
    out.spot = vec2<f32>(corner.x, corner.y * (1.0 + half));
    out.color = light;
    out.half = half;
    return out;
}

fn placed(out: StarVertex, world_from_local: mat4x4<f32>) -> StarVertex {
    var moved = out;
    let shell = length(world_from_local[0].xyz);
    let clip = position_world_to_clip(world_from_local[3].xyz + out.position.xyz * shell);
    moved.position = vec4<f32>(clip.x, clip.y, 0.0, clip.w);
    return moved;
}

fn star(vertex: UncompressedVertex, world_from_local: mat4x4<f32>, corner: vec2<f32>) -> StarVertex {
    let built = -vertex.normal;
    let range = max(vertex.uv_b.x, 0.0001);
    let far = built * range - stars.drift.xyz;
    let now = max(length(far), 0.0001);
    let was = shine(vertex.uv_b.y);
    let flux = vertex.uv_b.y - 2.0 * log(now / range) / LOG10;
    let lit = shine(flux);
    let sized = vec2<f32>(lit.x, lit.y / was.y);

    let seen = see(normalize((world_from_local * vec4<f32>(far / now, 0.0)).xyz));
    let size = sized.x;
    let reach = min(0.5 * stars.params.y * seen.turn / (now * seen.factor), STREAK_MOST);
    let near = 1.0 - smoothstep(SIGHT - SIGHT_FADE, SIGHT, now);
    let share = stars.params.z * near;
    let day = smoothstep(DAY_FLUX, DAY_FLUX + 1.0, flux);
    let sky = max(day, lit_by_night(flux, seen.dir));
    let glow = vertex.color.rgb * (sized.y * sky * share) * beamed(seen.factor, reach / size);
    return emit(seen, corner, size, reach, vec4<f32>(glow, vertex.color.a));
}

fn dust(vertex: UncompressedVertex, corner: vec2<f32>) -> StarVertex {
    let axis = stars.heading.xyz;
    let index = u32(vertex.uv_b.x);
    let run = hash(index * 4u) + stars.flow.x;
    let lap = u32(i32(floor(run)) & 1023);
    let key = (index + MOTES * (lap + 1u)) * 4u;
    let t = fract(run);
    let reach = mix(TUBE_NEAR * TUBE_NEAR, TUBE_FAR * TUBE_FAR, hash(key + 1u));
    let turn = TAU * hash(key + 2u);
    let side = aside(axis);
    let across = (side * cos(turn) + cross(axis, side) * sin(turn)) * sqrt(reach);
    let offset = axis * mix(DEPTH_AHEAD, DEPTH_BEHIND, t) + across;
    let thin = clamp((stars.flow.z * 1.5 - hash(key + 3u)) * 2.0, 0.0, 1.0);
    let light = window(t) * thin / (1.0 + dot(offset, offset));

    let speed = stars.flow.y * (DEPTH_AHEAD - DEPTH_BEHIND);
    let before = see(normalize(offset + axis * (speed * EXPOSURE)));
    let now = see(normalize(offset));
    let swept = now.dir - before.dir;
    let span = length(swept);
    var seen = now;
    seen.dir = normalize(before.dir + now.dir);
    seen.along = select(now.along, swept / max(span, 0.000001), span > 0.000001);
    let half = clamp(0.5 * span, GRAIN * GRAIN_ASPECT, GRAIN_REACH);
    let moving = smoothstep(0.0, GRAIN_PACE, span);
    let grey = GRAIN_LIGHT * light * moving * stars.params.x * stars.params.z;
    return emit(seen, corner, GRAIN, half, vec4<f32>(vec3<f32>(grey), vertex.color.a));
}

@vertex
fn vertex(vertex_in: Vertex) -> StarVertex {
    let vertex = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    let corner = vertex.uv * 2.0 - vec2<f32>(1.0, 1.0);
    if (vertex.uv_b.y < DUST * 0.5) {
        return placed(dust(vertex, corner), world_from_local);
    }
    return placed(star(vertex, world_from_local, corner), world_from_local);
}

@fragment
fn fragment(in: StarVertex) -> @location(0) vec4<f32> {
    let q = vec2<f32>(in.spot.x, max(abs(in.spot.y) - in.half, 0.0));
    let core = clamp(1.0 - dot(q, q), 0.0, 1.0);
    let shape = pow(core, in.color.a);
    return vec4<f32>(in.color.rgb * shape, 0.0);
}
