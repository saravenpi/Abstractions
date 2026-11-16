import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Sky {
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    sun: vec4<f32>,
    moon: vec4<f32>,
    cloud: vec4<f32>,
    params: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: Sky;

const SUN_RADIUS2: f32 = 0.00125;
const MOON_RADIUS: f32 = 0.045;

fn hash21(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(123.34, 456.21));
    q = q + dot(q, q + vec2<f32>(45.32, 45.32));
    return fract(q.x * q.y);
}

fn hash31(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
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

fn fbm(p: vec2<f32>) -> f32 {
    var total = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var i = 0; i < 3; i = i + 1) {
        total = total + amplitude * value_noise(q);
        q = q * 2.03 + vec2<f32>(17.1, 9.2);
        amplitude = amplitude * 0.5;
    }
    return total;
}

fn star_field(v: vec3<f32>, time: f32) -> f32 {
    let p = v * 48.0;
    let cell = floor(p);
    let cell_pos = p - cell;
    let h = hash31(cell);
    let offset = vec3<f32>(0.4 + 0.2 * fract(h * 7.13), 0.4 + 0.2 * fract(h * 13.7), 0.4 + 0.2 * fract(h * 3.31));
    let size = 0.18 + 0.12 * fract(h * 31.0);
    let shape = 1.0 - smoothstep(0.0, size, length(cell_pos - offset));
    let twinkle = 0.75 + 0.25 * sin(time * 2.5 + h * 60.0);
    return step(0.9, h) * shape * twinkle;
}

fn clouds(v: vec3<f32>, time: f32, cover: f32, light_dir: vec3<f32>) -> vec2<f32> {
    let height = max(v.y, 0.0);
    let plane = v.xz / (height + 0.12);
    let wind = vec2<f32>(0.02, 0.008) * time;
    let p = floor(plane * 16.0) / 16.0 * 2.2 + wind;
    let n = fbm(p);
    let n_toward_light = fbm(p + light_dir.xz * 0.12);
    let threshold = mix(0.72, 0.4, cover);
    let density = smoothstep(threshold, threshold + 0.07, n);
    let chunky = floor(density * 3.0 + 0.5) / 3.0;
    let lit = clamp(0.6 + (n - n_toward_light) * 8.0, 0.0, 1.0);
    let fade = smoothstep(0.03, 0.25, v.y);
    return vec2<f32>(chunky * fade, lit);
}

fn moon_disc(v: vec3<f32>, m: vec3<f32>) -> vec4<f32> {
    let t = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), m));
    let b = cross(m, t);
    let q = vec2<f32>(dot(v, t), dot(v, b)) / MOON_RADIUS;
    let r2 = dot(q, q);
    if (dot(v, m) <= 0.0 || r2 >= 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let n = vec3<f32>(q, sqrt(1.0 - r2));
    let blotch = value_noise(q * 2.2 + vec2<f32>(5.0, 3.0)) * 0.65 + value_noise(q * 5.0) * 0.35;
    let maria = smoothstep(0.42, 0.62, blotch);
    let albedo = mix(vec3<f32>(0.92, 0.94, 1.0), vec3<f32>(0.5, 0.56, 0.68), maria * 0.8);
    let lit = smoothstep(-0.1, 0.3, dot(n, normalize(vec3<f32>(0.5, 0.2, 0.85))));
    let edge = 1.0 - smoothstep(0.9, 1.0, r2);
    return vec4<f32>(albedo * mix(0.1, 1.0, lit), edge);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = normalize(in.world_position.xyz - view.world_position);
    let sun_dir = sky.sun.xyz;
    let moon_dir = sky.moon.xyz;
    let discs = sky.params.w;
    let above_horizon = smoothstep(-0.01, 0.02, v.y);

    var col = mix(sky.horizon.rgb, sky.zenith.rgb, pow(clamp(v.y, 0.0, 1.0), 0.5));

    let star = star_field(v, sky.params.x) * sky.params.y * smoothstep(0.02, 0.2, v.y);
    col = col + vec3<f32>(0.9, 0.95, 1.0) * star * 1.5;

    let sun_visible = smoothstep(-0.15, 0.05, sun_dir.y) * discs * above_horizon;
    let sun_col = mix(vec3<f32>(1.0, 0.5, 0.2), vec3<f32>(1.0, 0.95, 0.8), smoothstep(0.0, 0.4, sun_dir.y));
    let sun_dist = 1.0 - dot(v, sun_dir);
    let sun_glow = exp(-sun_dist * 45.0) * 0.55 + exp(-sun_dist * 6.0) * 0.18;
    let sun_disc = 1.0 - smoothstep(SUN_RADIUS2 * 0.7, SUN_RADIUS2, sun_dist);
    col = col + sun_col * sun_glow * sun_visible;
    col = mix(col, sun_col * 5.0, sun_disc * sun_visible);

    let moon_visible = smoothstep(-0.15, 0.05, moon_dir.y) * discs * above_horizon;
    let moon_dist = 1.0 - dot(v, moon_dir);
    col = col + vec3<f32>(0.3, 0.4, 0.65) * exp(-moon_dist * 70.0) * 0.35 * moon_visible;
    let moon = moon_disc(v, moon_dir);
    col = mix(col, moon.rgb * 1.6, moon.a * moon_visible);

    if (sky.params.z > 0.001) {
        let cloud = clouds(v, sky.params.x, sky.params.z, sun_dir);
        let shaded = mix(sky.cloud.rgb * 0.55, sky.cloud.rgb * 1.05, cloud.y);
        col = mix(col, shaded, cloud.x);
    }

    return vec4<f32>(col, 1.0);
}
