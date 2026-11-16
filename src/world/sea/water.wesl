import bevy_pbr::render::forward_io::{Vertex, VertexOutput, decompress_vertex};
import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::mesh_view_bindings::view;
import bevy_pbr::render::view_transformations::{position_world_to_clip, depth_ndc_to_view_z};
@if(DEPTH_PREPASS)
import bevy_pbr::prepass::utils::prepass_depth;

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
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> water: Water;

const GRAVITY: f32 = 9.81;
const FADE_START: f32 = 48.0;
const FADE_END: f32 = 90.0;
const DEEP: f32 = 12.0;
const DETAIL_CELLS: f32 = 2.5;
const SLOPE_GAIN: f32 = 2.5;
const SHALLOW_COLOR: vec3<f32> = vec3<f32>(0.03, 0.55, 0.48);
const DEEP_COLOR: vec3<f32> = vec3<f32>(0.002, 0.04, 0.16);
const FOAM_COLOR: vec3<f32> = vec3<f32>(0.95, 0.98, 1.0);
const MOON_COLOR: vec3<f32> = vec3<f32>(0.55, 0.68, 1.0);

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q = q + dot(q, q + vec2<f32>(45.32, 45.32));
    return fract(q.x * q.y);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0, 3.0) - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
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

fn ripple(dir: vec2<f32>, k: f32, slope: f32, p: vec2<f32>, t: f32, seed: f32) -> vec2<f32> {
    let phase = k * dot(dir, p) - sqrt(GRAVITY * k) * t + seed;
    return dir * (slope * cos(phase));
}

fn surface_slope(p: vec2<f32>, t: f32, wind: f32) -> vec2<f32> {
    let dir = water.wave_a.xy;
    let gain = 0.5 + wind;
    return waves(p, t).yz * SLOPE_GAIN
        + ripple(rotate(dir, 0.7), 3.3, 0.09 * gain, p, t, 2.1)
        + ripple(rotate(dir, -1.1), 5.7, 0.06 * gain, p, t, 4.3);
}

fn sky_color(d: vec3<f32>) -> vec3<f32> {
    return mix(water.horizon.rgb, water.zenith.rgb, pow(clamp(d.y, 0.0, 1.0), 0.5));
}

fn glitter(r: vec3<f32>, light: vec3<f32>, cell: vec2<f32>, t: f32) -> f32 {
    let s = dot(r, light);
    let twinkle = 0.6 + 0.4 * hash21(cell + vec2<f32>(floor(t * 2.0), 3.0));
    let sparks = step(0.55, smoothstep(0.9935, 0.9985, s) * twinkle);
    let path = smoothstep(0.97, 1.0, s) * 0.25;
    return sparks * 3.0 + path;
}

fn column_depth(frag: vec4<f32>, world: vec3<f32>) -> f32 {
    var depth = DEEP;
    @if(DEPTH_PREPASS) {
        let raw = prepass_depth(frag, 0u);
        if (raw > 0.0) {
            let ratio = depth_ndc_to_view_z(raw) / depth_ndc_to_view_z(frag.z);
            let scene_y = view.world_position.y + (world.y - view.world_position.y) * ratio;
            depth = clamp(world.y - scene_y, 0.0, DEEP);
        }
    }
    return depth;
}

fn foam_amount(p: vec2<f32>, depth: f32, t: f32) -> f32 {
    let n = value_noise(p * 0.8 + vec2<f32>(t * 0.05, -t * 0.03));
    let fine = value_noise(p * 2.6 - vec2<f32>(t * 0.2, 0.0));
    let pulse = 0.5 + 0.5 * sin(t * 1.3 + n * 6.0);
    let limit = 0.6 * (0.45 + 0.55 * pulse) + (n - 0.5) * 0.3;
    let wash = step(depth, limit);
    let reach = step(0.6, depth) * (1.0 - smoothstep(1.6, 2.5, depth));
    let band = fract(depth * 0.8 + t * 0.2 + n * 0.5);
    let lines = step(band, 0.16) * step(0.4, fine) * reach;
    return max(wash, lines);
}

fn crest_flecks(p: vec2<f32>, t: f32, wind: f32) -> f32 {
    let total = water.wave_a.w + water.wave_b.w + water.wave_c.w + water.wave_d.w;
    let crest = waves(p, t).x / max(total, 0.001);
    let grain = value_noise(p * 3.5 + vec2<f32>(t * 0.4, 0.0));
    return step(0.6, crest) * step(0.68, grain) * smoothstep(0.5, 0.9, wind);
}

fn under_surface(v: vec3<f32>, n: vec3<f32>, p: vec2<f32>, t: f32) -> vec4<f32> {
    let day = water.sun_color.w;
    let look = abs(dot(-v, n));
    let window = smoothstep(0.62, 0.74, look);
    let dim = water.fog.rgb * 0.7;
    let bright = (water.fog.rgb * 1.6 + vec3<f32>(0.04, 0.16, 0.16)) * (0.35 + 0.65 * day);
    let shimmer = step(0.78, value_noise(p * 0.9 + vec2<f32>(t * 0.35, t * 0.2))) * window * 0.35 * day;
    return vec4<f32>(mix(dim, bright, window) + vec3<f32>(shimmer), 0.92);
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let vertex = decompress_vertex(vertex_no_morph, vertex_no_morph.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    var world = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    let edge = max(abs(vertex.position.x), abs(vertex.position.z));
    let fade = 1.0 - smoothstep(FADE_START, FADE_END, edge);
    world.y = world.y + waves(world.xz, water.params.x).x * fade;
    out.world_position = world;
    out.position = position_world_to_clip(world.xyz);
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
    @if(VERTEX_OUTPUT_INSTANCE_INDEX)
    out.instance_index = vertex_no_morph.instance_index;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = water.params.x;
    let wind = water.params.y;
    let world = in.world_position.xyz;
    let to_eye = view.world_position - world;
    let dist = length(to_eye);
    let v = to_eye / max(dist, 0.001);
    let detail = 1.0 - smoothstep(60.0, 260.0, dist);
    let cell = floor(world.xz * DETAIL_CELLS);
    let slope = surface_slope((cell + vec2<f32>(0.5, 0.5)) / DETAIL_CELLS, t, wind) * detail;
    var n = normalize(vec3<f32>(-slope.x, 1.0, -slope.y));
    let below = v.y < 0.0;
    if (below) {
        n = -n;
    }

    var col: vec4<f32>;
    if (below) {
        col = under_surface(v, n, world.xz, t);
    } else {
        let day = water.sun_color.w;
        let light = mix(vec3<f32>(0.07, 0.10, 0.22), mix(vec3<f32>(1.0), water.sun_color.rgb, 0.4), day);
        let depth = column_depth(in.position, world);
        let shade = floor((1.0 - exp(-depth * 0.5)) * 6.0 + 0.5) / 6.0;
        let body = mix(SHALLOW_COLOR, DEEP_COLOR, shade) * light;

        let ndv = max(dot(n, v), 0.0);
        let fresnel = 0.05 + 0.95 * pow(1.0 - ndv, 4.0);
        let r = reflect(-v, n);
        var color = mix(body, sky_color(r), fresnel);
        let sun_glint = glitter(r, water.sun.xyz, cell, t) * water.sun.w;
        let moon_glint = glitter(r, water.moon.xyz, cell, t) * water.moon.w * 0.5;
        color = color + (water.sun_color.rgb * sun_glint + MOON_COLOR * moon_glint) * detail;

        let foam = max(foam_amount(world.xz, depth, t), crest_flecks(world.xz, t, wind) * detail);
        let foam_light = FOAM_COLOR * mix(0.3, 1.0, day) * mix(vec3<f32>(1.0), water.sun_color.rgb, 0.35);
        color = mix(color, foam_light, foam);

        let clear = mix(0.2, 0.95, shade);
        let opacity = mix(clear, 1.0, pow(1.0 - ndv, 3.0));
        col = vec4<f32>(color, max(opacity, foam));
    }

    let fog_amount = clamp((dist - water.params.z) / max(water.params.w - water.params.z, 1.0), 0.0, 1.0);
    return vec4<f32>(mix(col.rgb, water.fog.rgb, fog_amount), mix(col.a, 1.0, fog_amount));
}
