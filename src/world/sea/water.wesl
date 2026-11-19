import bevy_pbr::render::forward_io::{Vertex, decompress_vertex};
import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::mesh_view_bindings::view;
import bevy_pbr::render::view_transformations::{position_world_to_clip, depth_ndc_to_view_z};
import bevy_pbr::render::pbr_types::{PbrInput, pbr_input_new};
import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, calculate_view};
@if(DEPTH_PREPASS)
import bevy_pbr::prepass::utils::prepass_depth;
import package::render::daylight::{Place, relit, height_at, sky_gain};

struct Water {
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    fog: vec4<f32>,
    sun: vec4<f32>,
    sun_color: vec4<f32>,
    moon: vec4<f32>,
    wave_a: vec4<f32>,
    wave_b: vec4<f32>,
    wave_c: vec4<f32>,
    wave_d: vec4<f32>,
    params: vec4<f32>,
    map: vec4<f32>,
    globe: vec4<f32>,
    frame: vec4<f32>,
    shallow: vec4<f32>,
    deep: vec4<f32>,
    foam: vec4<f32>,
    far: vec4<f32>,
    blend: vec4<f32>,
}

struct SeaVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) chart: vec2<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> water: Water;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var floor_map: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var floor_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var<uniform> place: Place;

const GRAVITY: f32 = 9.81;
const FADE_START: f32 = 48.0;
const FADE_END: f32 = 90.0;
const DEEP: f32 = 12.0;
const DETAIL_CELLS: f32 = 2.5;
const SLOPE_GAIN: f32 = 2.5;
const MOON_COLOR: vec3<f32> = vec3<f32>(0.55, 0.68, 1.0);
const MAP_SPAN: f32 = 256.0;
const MAP_TEXEL: f32 = 2.0;
const MAP_RANGE: f32 = 20.0;
const MAP_LAND: f32 = 4.0;
const MAP_FADE_START: f32 = 104.0;
const MAP_FADE_END: f32 = 120.0;
const MIN_BED_SLOPE: f32 = 0.03;
const SHOAL_DEPTH: f32 = 2.5;
const NOISE_PERIOD: f32 = 512.0;
const EDGE_DEPTH: f32 = 0.05;
const POND_CALM: f32 = 0.75;
const SWASH_REACH: f32 = 0.9;
const SWASH_BREATH: f32 = 0.15;
const SWASH_RATE: f32 = 1.05;
const SWASH_RAGGED: f32 = 0.3;
const LINE_FAR: f32 = 5.0;
const LINE_NEAR: f32 = 2.0;
const LINE_RATE: f32 = 0.1;
const LINE_HALF: f32 = 0.15;
const FLECK_DEPTH: f32 = 2.0;
const HAND_SOFT: f32 = 0.83;
const WAVE_TOP: f32 = 0.5;
const TAU: f32 = 6.2831853;
const KEEP_START: f32 = 0.15;
const KEEP_END: f32 = 0.5;
const SPARK_START: f32 = 0.4;
const SPARK_END: f32 = 1.2;
const SHELF_START: f32 = 1.1;
const SHELF_END: f32 = 1.4;
const MAP_LOW: f32 = 30.0;
const MAP_HIGH: f32 = 120.0;
const COARSE_ROUGHNESS: f32 = 0.95;
const COARSE_REFLECTANCE: f32 = 0.05;
const DAY_EDGE: vec2<f32> = vec2<f32>(-0.1, 0.3);
const SUN_UP_EDGE: vec2<f32> = vec2<f32>(-0.05, 0.12);

fn hash21(p: vec2<f32>) -> f32 {
    let q = bitcast<vec2<u32>>(vec2<i32>(p));
    var h = q.x * 0x27d4eb2du + q.y * 0x165667b1u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn lattice(i: vec2<f32>, period: f32) -> f32 {
    return hash21(i - floor(i / period) * period);
}

fn value_noise(p: vec2<f32>, freq: f32) -> f32 {
    let period = NOISE_PERIOD * freq;
    let x = (p - floor(p / NOISE_PERIOD) * NOISE_PERIOD) * freq;
    let i = floor(x);
    let f = x - i;
    let u = f * f * (vec2<f32>(3.0, 3.0) - 2.0 * f);
    let a = lattice(i, period);
    let b = lattice(i + vec2<f32>(1.0, 0.0), period);
    let c = lattice(i + vec2<f32>(0.0, 1.0), period);
    let d = lattice(i + vec2<f32>(1.0, 1.0), period);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn floor_tap(p: vec2<f32>) -> vec2<f32> {
    let uv = (p + vec2<f32>(0.5 * MAP_TEXEL)) / MAP_SPAN;
    let s = textureSampleLevel(floor_map, floor_sampler, uv, 0.0);
    let depth = (s.r * 65280.0 + s.g * 255.0) / 65535.0 * MAP_RANGE - MAP_LAND;
    return vec2<f32>(depth, s.b);
}

fn floor_valid(p: vec2<f32>) -> f32 {
    let o = abs(p - water.map.xy);
    return 1.0 - smoothstep(MAP_FADE_START, MAP_FADE_END, max(o.x, o.y));
}

fn shore(p: vec2<f32>) -> vec4<f32> {
    let here = floor_tap(p);
    let dx = floor_tap(p + vec2<f32>(MAP_TEXEL, 0.0)).x - here.x;
    let dz = floor_tap(p + vec2<f32>(0.0, MAP_TEXEL)).x - here.x;
    let slope = length(vec2<f32>(dx, dz)) / MAP_TEXEL;
    let near = floor_valid(p);
    return vec4<f32>(here.x / max(slope, MIN_BED_SLOPE), here.x, here.y * near, near);
}

fn lift(p: vec2<f32>, radius: f32) -> vec3<f32> {
    let angle = length(p) / radius;
    let reach = sin(angle) / max(angle, 0.000001);
    let half = sin(0.5 * angle);
    return vec3<f32>(p.x * reach, -2.0 * radius * half * half, p.y * reach);
}

fn rotate(d: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c);
}

fn wave_term(w: vec4<f32>, p: vec2<f32>, t: f32, seed: f32) -> vec3<f32> {
    let phase = w.z * dot(w.xy, p) - sqrt(GRAVITY * w.z) * t + seed;
    let slope = w.w * w.z * cos(phase);
    return vec3<f32>(w.w * sin(phase), slope * w.x, slope * w.y);
}

fn waves(p: vec2<f32>, t: f32) -> vec3<f32> {
    return wave_term(water.wave_a, p, t, 0.0)
        + wave_term(water.wave_b, p, t, 1.7)
        + wave_term(water.wave_c, p, t, 3.1)
        + wave_term(water.wave_d, p, t, 4.9);
}

fn keep(k: f32, foot: f32) -> f32 {
    return 1.0 - smoothstep(KEEP_START, KEEP_END, foot * k / TAU);
}

fn wave_slope(w: vec4<f32>, p: vec2<f32>, t: f32, seed: f32, foot: f32) -> vec2<f32> {
    return wave_term(w, p, t, seed).yz * keep(w.z, foot);
}

fn ripple(dir: vec2<f32>, k: f32, slope: f32, p: vec2<f32>, t: f32, seed: f32) -> vec2<f32> {
    let phase = k * dot(dir, p) - sqrt(GRAVITY * k) * t + seed;
    return dir * (slope * cos(phase));
}

fn surface_slope(p: vec2<f32>, t: f32, wind: f32, foot: f32) -> vec2<f32> {
    let dir = water.wave_a.xy;
    let gain = 0.5 + wind;
    let swell = wave_slope(water.wave_a, p, t, 0.0, foot)
        + wave_slope(water.wave_b, p, t, 1.7, foot)
        + wave_slope(water.wave_c, p, t, 3.1, foot)
        + wave_slope(water.wave_d, p, t, 4.9, foot);
    return swell * SLOPE_GAIN
        + ripple(rotate(dir, 0.7), 3.3, 0.09 * gain, p, t, 2.1) * keep(3.3, foot)
        + ripple(rotate(dir, -1.1), 5.7, 0.06 * gain, p, t, 4.3) * keep(5.7, foot);
}

fn sky_color(rise: f32) -> vec3<f32> {
    return mix(water.horizon.rgb, water.zenith.rgb, pow(clamp(rise, 0.0, 1.0), 0.5));
}

fn glitter(r: vec3<f32>, light: vec3<f32>, cell: vec2<f32>, t: f32) -> f32 {
    let s = dot(r, light);
    let twinkle = 0.6 + 0.4 * hash21(cell + vec2<f32>(floor(t * 2.0), 3.0));
    let sparks = step(0.55, smoothstep(0.9935, 0.9985, s) * twinkle);
    let path = smoothstep(0.97, 1.0, s) * 0.25;
    return sparks * 3.0 + path;
}

fn column_depth(frag: vec4<f32>, eye_above: f32) -> f32 {
    var depth = DEEP;
    @if(DEPTH_PREPASS) {
        let raw = prepass_depth(frag, 0u);
        if (raw > 0.0) {
            let ratio = depth_ndc_to_view_z(raw) / depth_ndc_to_view_z(frag.z);
            depth = clamp(eye_above * (ratio - 1.0), 0.0, DEEP);
        }
    }
    return depth;
}

fn shelved(depth: f32) -> f32 {
    return mix(depth, DEEP, smoothstep(SHELF_START, SHELF_END, depth));
}

fn swash(p: vec2<f32>, d: f32, pixel: f32, t: f32) -> f32 {
    let n = value_noise(p + vec2<f32>(t * 0.1, -t * 0.06), 0.25);
    let ragged = value_noise(p - vec2<f32>(t * 0.05, t * 0.08), 0.75);
    let reach = SWASH_REACH + SWASH_BREATH * sin(t * SWASH_RATE + n * 3.0) + (ragged - 0.5) * SWASH_RAGGED;
    let soft = clamp(pixel, 0.15, 0.8);
    return (1.0 - smoothstep(reach - soft, reach + soft, d)) * smoothstep(0.15, 0.35, ragged);
}

fn foam_line(d: f32, phase: f32, soft: f32) -> f32 {
    let at = mix(LINE_FAR, LINE_NEAR, phase);
    let life = smoothstep(0.0, 0.25, phase) * (1.0 - smoothstep(0.7, 1.0, phase));
    return (1.0 - smoothstep(LINE_HALF, LINE_HALF + soft, abs(d - at))) * life;
}

fn foam_lines(p: vec2<f32>, d: f32, pixel: f32, t: f32) -> f32 {
    let phase = t * LINE_RATE + value_noise(p, 0.125) * 0.4;
    let dash = smoothstep(0.4, 0.55, value_noise(p, 1.5));
    let soft = clamp(pixel, 0.12, 0.6);
    let lines = max(foam_line(d, fract(phase), soft), foam_line(d, fract(phase + 0.5), soft));
    return lines * dash * (1.0 - smoothstep(0.6, 1.8, pixel));
}

fn shore_foam(p: vec2<f32>, bed: vec4<f32>, pixel: f32, t: f32) -> f32 {
    let sea = bed.w * (1.0 - smoothstep(0.4, 0.6, bed.z));
    return max(swash(p, bed.x, pixel, t), foam_lines(p, bed.x, pixel, t)) * sea;
}

fn crest_flecks(p: vec2<f32>, t: f32, wind: f32) -> f32 {
    let total = water.wave_a.w + water.wave_b.w + water.wave_c.w + water.wave_d.w;
    let crest = waves(p, t).x / max(total, 0.001);
    let grain = value_noise(p + vec2<f32>(t * 0.114, 0.0), 3.5);
    return step(0.6, crest) * step(0.68, grain) * smoothstep(0.5, 0.9, wind);
}

fn under_surface(v: vec3<f32>, n: vec3<f32>, p: vec2<f32>, t: f32) -> vec4<f32> {
    let day = water.sun_color.w;
    let look = abs(dot(-v, n));
    let window = smoothstep(0.62, 0.74, look);
    let dim = water.fog.rgb * 0.7;
    let bright = (water.fog.rgb * 1.6 + vec3<f32>(0.04, 0.16, 0.16)) * (0.35 + 0.65 * day);
    let shimmer = step(0.78, value_noise(p + vec2<f32>(t * 0.4, t * 0.23), 0.875)) * window * 0.35 * day;
    return vec4<f32>(mix(dim, bright, window) + vec3<f32>(shimmer), 0.92);
}

fn coarse_sea(world: vec3<f32>, up: vec3<f32>, frag: vec4<f32>) -> vec3<f32> {
    var input: PbrInput = pbr_input_new();
    input.material.base_color = vec4<f32>(water.far.rgb, 1.0);
    input.material.emissive = vec4<f32>(water.deep.rgb * water.shallow.w, 0.0);
    input.material.perceptual_roughness = COARSE_ROUGHNESS;
    input.material.reflectance = vec3<f32>(COARSE_REFLECTANCE);
    input.frag_coord = frag;
    input.world_position = vec4<f32>(world, 1.0);
    input.world_normal = up;
    input.N = up;
    input.V = calculate_view(vec4<f32>(world, 1.0), false);
    return relit(input, apply_pbr_lighting(input), place).rgb;
}

fn daytime(world: vec3<f32>) -> vec3<f32> {
    if (place.air.z <= 0.0) {
        return vec3<f32>(water.sun_color.w, water.sun.w, 1.0);
    }
    let here = height_at(place, world).x;
    let eye = height_at(place, view.world_position).x;
    let day = smoothstep(DAY_EDGE.x, DAY_EDGE.y, here) - smoothstep(DAY_EDGE.x, DAY_EDGE.y, eye);
    let up = smoothstep(SUN_UP_EDGE.x, SUN_UP_EDGE.y, here) - smoothstep(SUN_UP_EDGE.x, SUN_UP_EDGE.y, eye);
    let gain = sky_gain(place, world, view.world_position);
    return vec3<f32>(saturate(water.sun_color.w + day), saturate(water.sun.w + up), gain);
}

fn afar(to_eye: vec3<f32>, eye_height: f32) -> f32 {
    let reach = smoothstep(water.blend.x, water.blend.y, length(to_eye));
    return max(reach, smoothstep(water.blend.z, water.blend.w, eye_height));
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> SeaVertex {
    var out: SeaVertex;
    let vertex = decompress_vertex(vertex_no_morph, vertex_no_morph.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    let p = water.map.zw + vertex.position.xz;
    let radius = water.globe.w;
    let edge = max(abs(vertex.position.x), abs(vertex.position.z));
    let fade = 1.0 - smoothstep(FADE_START, FADE_END, edge);
    let bed = floor_tap(p);
    let calm = mix(1.0, smoothstep(0.0, SHOAL_DEPTH, bed.x) * (1.0 - bed.y), floor_valid(p));
    let still = lift(p, radius);
    let outward = normalize(still + vec3<f32>(0.0, radius, 0.0));
    let local = still + outward * (waves(p, water.params.x).x * fade * calm);
    let world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(local, 1.0));
    out.world_position = world;
    out.position = position_world_to_clip(world.xyz);
    out.chart = p;
    return out;
}

@fragment
fn fragment(in: SeaVertex) -> @location(0) vec4<f32> {
    let t = water.params.x;
    let wind = water.params.y;
    let world = in.world_position.xyz;
    let p = in.chart;
    let up = normalize(world - water.globe.xyz);
    let east = normalize(water.frame.xyz - up * dot(water.frame.xyz, up));
    let south = cross(east, up);
    let to_eye = view.world_position - world;
    let dist = length(to_eye);
    let v = to_eye / max(dist, 0.001);
    let eye_height = length(view.world_position - water.globe.xyz) - water.globe.w;
    let detail = 1.0 - smoothstep(60.0, 260.0, dist);
    let cell = floor(p * DETAIL_CELLS);
    let bed = shore(p);
    let pixel = fwidth(bed.x);
    let foot = max(length(dpdx(p)), length(dpdy(p)));
    let fine = detail * (1.0 - smoothstep(SPARK_START, SPARK_END, foot));
    let still = 1.0 - POND_CALM * bed.z;
    let slope = surface_slope((cell + vec2<f32>(0.5, 0.5)) / DETAIL_CELLS, t, wind, foot) * detail * still;
    var n = normalize(up - east * slope.x - south * slope.y);
    let rise = dot(v, up);
    let below = rise < 0.0;
    if (below) {
        n = -n;
    }

    var col: vec4<f32>;
    if (below) {
        col = under_surface(v, n, p, t);
    } else {
        let time = daytime(world);
        let day = time.x;
        let light = mix(vec3<f32>(0.07, 0.10, 0.22), mix(vec3<f32>(1.0), water.sun_color.rgb, 0.4), day);
        let sag = max(water.globe.w - length(world - water.globe.xyz), 0.0);
        let column = min(column_depth(in.position, rise * dist) + sag, DEEP);
        let shelf = smoothstep(SHELF_START, SHELF_END, column);
        let mapped = bed.w * (1.0 - smoothstep(MAP_LOW, MAP_HIGH, eye_height));
        let depth = mix(shelved(column), shelved(max(bed.y, 0.0)), mapped);
        let shade = floor((1.0 - exp(-depth * 0.5)) * 6.0 + 0.5) / 6.0;
        let body = mix(water.shallow.rgb, water.deep.rgb, shade) * (light + vec3<f32>(water.shallow.w));

        let ndv = max(dot(n, v), 0.0);
        let fresnel = (0.05 + 0.95 * pow(1.0 - ndv, 4.0)) * water.deep.w;
        let r = reflect(-v, n);
        var color = mix(body, sky_color(dot(r, up)) * time.z, fresnel);
        let sun_glint = glitter(r, water.sun.xyz, cell, t) * time.y;
        let moon_glint = glitter(r, water.moon.xyz, cell, t) * water.moon.w * 0.5;
        color = color + (water.sun_color.rgb * sun_glint + MOON_COLOR * moon_glint) * fine * water.deep.w;

        let open = mix(1.0, smoothstep(FLECK_DEPTH, FLECK_DEPTH + 1.0, bed.y), bed.w) * (1.0 - bed.z);
        let flecks = crest_flecks(p, t, wind) * fine * open;
        let foam = max(shore_foam(p, bed, pixel, t), flecks);
        let foam_light = water.foam.rgb * mix(0.07, 1.0, day) * mix(vec3<f32>(1.0), water.sun_color.rgb, 0.35);
        color = mix(color, foam_light, foam);

        let clear = select(mix(0.2, 0.95, shade), 1.0, shade >= 1.0);
        let opacity = max(mix(clear, 1.0, pow(1.0 - ndv, 3.0)), water.foam.w);
        col = vec4<f32>(color, max(opacity, foam) * smoothstep(0.0, EDGE_DEPTH, column));
        let turned = afar(to_eye, eye_height);
        if (turned > 0.0) {
            col = mix(col, vec4<f32>(coarse_sea(world, up, in.position), shelf), turned);
        }
    }

    let fog_amount = clamp((dist - water.params.z) / max(water.params.w - water.params.z, 1.0), 0.0, 1.0);
    let near = 1.0 - smoothstep(water.frame.w * HAND_SOFT, water.frame.w, dist);
    let far_side = f32(below) * step(WAVE_TOP, eye_height);
    let alpha = mix(col.a, 1.0, fog_amount) * near * (1.0 - far_side);
    if (alpha <= 0.0) {
        discard;
    }
    return vec4<f32>(mix(col.rgb, water.fog.rgb, fog_amount), alpha);
}
