const EDGE: f32 = 0.12;
const THRESHOLD: vec2<f32> = vec2<f32>(0.72, 0.4);
const OCTAVE_SCALE: f32 = 2.03;
const OCTAVE_SHIFT: vec3<f32> = vec3<f32>(17.1, 9.2, 4.7);
const OCTAVES: i32 = 4;
const LONE: vec2<f32> = vec2<f32>(0.3, 0.8);
const NIGHT_SIDE: f32 = 0.04;
const GRAZE: f32 = 0.08;
const LIMB: f32 = 0.3;
const IGN: vec3<f32> = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
const IGN_SHIFT: vec2<f32> = vec2<f32>(17.0, 31.0);

struct Puff {
    density: f32,
    thick: f32,
    relief: f32,
}

fn dither(texel: vec2<f32>) -> f32 {
    let p = texel + IGN_SHIFT;
    return fract(IGN.z * fract(p.x * IGN.x + p.y * IGN.y));
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

fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var total = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var i = 0; i < octaves; i = i + 1) {
        total = total + amplitude * space_noise(q);
        q = q * OCTAVE_SCALE + OCTAVE_SHIFT;
        amplitude = amplitude * 0.5;
    }
    return total;
}

fn threshold(cover: f32) -> f32 {
    return mix(THRESHOLD.x, THRESHOLD.y, cover);
}

fn density(noise: f32, cover: f32) -> f32 {
    let start = threshold(cover);
    return smoothstep(start, start + EDGE, noise);
}

fn broad_cover(map: texture_cube<f32>, map_sampler: sampler, dir: vec3<f32>, toward_second: f32) -> f32 {
    let held = textureSampleLevel(map, map_sampler, dir, 0.0);
    return mix(held.r, held.g, toward_second);
}

fn puff(spot: vec3<f32>, sunward: vec3<f32>, facing: f32, cover: f32) -> Puff {
    let here = fbm(spot, OCTAVES);
    let toward_light = fbm(spot + sunward * 0.3, 2);
    let limb = LIMB * pow(1.0 - abs(facing), 4.0);
    let dense = density(here + limb, cover);
    let thick = 1.0 - pow(1.0 - dense, 1.0 / max(abs(facing), GRAZE));
    let relief = clamp(0.6 + (here - toward_light * 1.25) * 8.0, 0.0, 1.0);
    return Puff(dense, thick, relief);
}

fn lone(cloud: Puff, cover: f32) -> f32 {
    return mix(smoothstep(LONE.x, LONE.y, cloud.density), 1.0, clamp(cover, 0.0, 1.0));
}

fn day_side(color: vec3<f32>, n: vec3<f32>, light: vec3<f32>) -> vec3<f32> {
    return color * mix(NIGHT_SIDE, 1.0, smoothstep(-0.2, 0.25, dot(n, light)));
}

fn shade(base: vec3<f32>, cloud: Puff, above: bool) -> vec3<f32> {
    let body = select(mix(1.0, 0.72, cloud.density), mix(0.82, 1.05, cloud.density), above);
    return mix(base * 0.55, base * 1.05, cloud.relief) * body;
}
