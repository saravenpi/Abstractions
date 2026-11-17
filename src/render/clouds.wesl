import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct Clouds {
    light: vec4<f32>,
    color: vec4<f32>,
    haze: vec4<f32>,
    params: vec4<f32>,
    fixed_x: vec4<f32>,
    fixed_y: vec4<f32>,
    fixed_z: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> clouds: Clouds;

const BLOCKS: f32 = 4.0;
const WIND: vec3<f32> = vec3<f32>(0.011, 0.0, 0.0045);
const SPACE_COLOR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
const NIGHT_SIDE: f32 = 0.04;

const BAYER: array<f32, 16> = array<f32, 16>(
    0.0, 8.0, 2.0, 10.0,
    12.0, 4.0, 14.0, 6.0,
    3.0, 11.0, 1.0, 9.0,
    15.0, 7.0, 13.0, 5.0
);

fn dither(texel: vec2<i32>) -> f32 {
    var cells = BAYER;
    let cell = (texel.y & 3) * 4 + (texel.x & 3);
    return (cells[cell] + 0.5) / 16.0;
}

fn hash31(p: vec3<f32>) -> f32 {
    let q = bitcast<vec3<u32>>(vec3<i32>(p));
    var h = q.x * 0x27d4eb2du + q.y * 0x165667b1u + q.z * 0x9e3779b1u;
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = (h ^ (h >> 12u)) * 0x297a2d39u;
    h = h ^ (h >> 15u);
    return f32(h >> 8u) / 16777216.0;
}

fn space_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (vec3<f32>(3.0) - 2.0 * f);
    let low = mix(
        mix(hash31(i), hash31(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
        mix(hash31(i + vec3<f32>(0.0, 1.0, 0.0)), hash31(i + vec3<f32>(1.0, 1.0, 0.0)), u.x),
        u.y
    );
    let high = mix(
        mix(hash31(i + vec3<f32>(0.0, 0.0, 1.0)), hash31(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
        mix(hash31(i + vec3<f32>(0.0, 1.0, 1.0)), hash31(i + vec3<f32>(1.0, 1.0, 1.0)), u.x),
        u.y
    );
    return mix(low, high, u.z);
}

fn fbm(p: vec3<f32>) -> f32 {
    var total = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var i = 0; i < 3; i = i + 1) {
        total = total + amplitude * space_noise(q);
        q = q * 2.03 + vec3<f32>(17.1, 9.2, 4.7);
        amplitude = amplitude * 0.5;
    }
    return total;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let to_fixed = mat3x3<f32>(clouds.fixed_x.xyz, clouds.fixed_y.xyz, clouds.fixed_z.xyz);
    let n = normalize(in.world_normal);
    let light = clouds.light.xyz;
    let air = clouds.light.w;
    let cover = clouds.color.w;
    let whiteout = clouds.haze.w;
    let time = clouds.params.x;

    let spot = to_fixed * n * clouds.params.y + WIND * time;
    let p = floor(spot * BLOCKS) / BLOCKS;
    let here = fbm(p);
    let toward_light = fbm(p + to_fixed * light * 0.3);
    let threshold = mix(0.72, 0.4, cover);
    let density = smoothstep(threshold, threshold + 0.07, here);
    let chunky = floor(density * 3.0 + 0.5) / 3.0;

    let dist = length(in.world_position.xyz - view.world_position);
    let far = smoothstep(0.35, 1.0, dist / max(clouds.params.z, 1.0));
    let alpha = max(chunky * (1.0 - 0.75 * far * air), whiteout);
    if (alpha <= dither(vec2<i32>(in.position.xy))) {
        discard;
    }

    let relief = clamp(0.6 + (here - toward_light) * 8.0, 0.0, 1.0);
    let day = smoothstep(-0.2, 0.25, dot(n, light));
    let from_space = SPACE_COLOR * mix(NIGHT_SIDE, 1.0, day);
    let base = mix(from_space, clouds.color.rgb, air);
    var col = mix(base * 0.55, base * 1.05, relief);
    col = mix(col, clouds.haze.rgb, far * air * 0.8);
    col = mix(col, clouds.color.rgb, whiteout);
    return vec4<f32>(col, 1.0);
}
