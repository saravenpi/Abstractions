const EDGE: f32 = 0.12;
const THRESHOLD: vec2<f32> = vec2<f32>(0.72, 0.25);
const OCTAVE_SCALE: f32 = 2.03;
const OCTAVE_SHIFT: vec3<f32> = vec3<f32>(17.1, 9.2, 4.7);
const OCTAVES: i32 = 4;
const LONE: vec2<f32> = vec2<f32>(0.3, 0.8);
const NIGHT_SIDE: f32 = 0.04;
const SHARP: vec2<f32> = vec2<f32>(0.25, 0.5);
const MEAN: f32 = 0.46875;
const SPREAD: f32 = 0.1065;
const TAIL: vec2<f32> = vec2<f32>(1.469, 0.1396);
const CUT: f32 = 0.066;
const TALL: f32 = 1.0;
const GRAZE: f32 = 0.02;
const CLAMP: f32 = 9.0;
const RELIEF_TOP: f32 = 0.85;
const SKEW: mat3x3<f32> = mat3x3<f32>(0.648638, 0.682114, -0.337622, -0.574003, 0.729721, 0.37152, 0.499789, -0.047186, 0.864861);
const IGN: vec3<f32> = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
const IGN_SHIFT: vec2<f32> = vec2<f32>(17.0, 31.0);

struct Puff {
    density: f32,
    relief: f32,
}

struct Spot {
    base: vec3<f32>,
    fine: vec3<f32>,
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

fn turned(v: vec3<f32>, c: f32, s: f32) -> vec3<f32> {
    return vec3<f32>(v.x * c - v.z * s, v.y, v.x * s + v.z * c);
}

fn spot_at(p: vec3<f32>, drift: vec4<f32>) -> Spot {
    return Spot(SKEW * turned(p, drift.x, drift.y), SKEW * turned(p, drift.z, drift.w));
}

fn fbm_part(spot: Spot, octaves: i32, read: i32, footprint: f32) -> f32 {
    var total = 0.5 * (space_noise(spot.base) - 0.5);
    var full = 0.25;
    var kept = 0.25;
    var amplitude = 0.25;
    var scale = OCTAVE_SCALE;
    var q = spot.fine * OCTAVE_SCALE + OCTAVE_SHIFT;
    for (var i = 1; i < octaves; i = i + 1) {
        let keep = select(0.0, 1.0 - smoothstep(SHARP.x, SHARP.y, footprint * scale), i < read);
        if (keep > 0.0) {
            total = total + amplitude * keep * (space_noise(q) - 0.5);
        }
        full = full + amplitude * amplitude;
        kept = kept + amplitude * amplitude * keep * keep;
        q = q * OCTAVE_SCALE + OCTAVE_SHIFT;
        amplitude = amplitude * 0.5;
        scale = scale * OCTAVE_SCALE;
    }
    return 0.5 - pow(0.5, f32(octaves + 1)) + total * sqrt(full / kept);
}

fn fbm_seen(spot: Spot, octaves: i32, footprint: f32) -> f32 {
    return fbm_part(spot, octaves, octaves, footprint);
}

fn threshold(cover: f32) -> f32 {
    return mix(THRESHOLD.x, THRESHOLD.y, cover);
}

fn lean(z: f32) -> f32 {
    return TAIL.x * z + TAIL.y * z * z * z;
}

fn cube_root(x: f32) -> f32 {
    return sign(x) * pow(max(abs(x), 1e-12), 1.0 / 3.0);
}

fn unlean(y: f32) -> f32 {
    let half = y / (2.0 * TAIL.y);
    let third = TAIL.x / (3.0 * TAIL.y);
    let root = sqrt(half * half + third * third * third);
    return cube_root(half + root) + cube_root(half - root);
}

fn graze(cover: f32, facing: f32) -> f32 {
    let f = clamp(abs(facing), GRAZE, 1.0);
    let crossed = 1.0 + TALL * sqrt(1.0 - f * f) / f;
    let z = (threshold(cover) + CUT - MEAN) / SPREAD;
    let y = clamp(lean(z), -CLAMP, CLAMP);
    let clear = -crossed * log(1.0 + exp(-y));
    let seen = clamp(clear - log(max(1.0 - exp(clear), 1e-6)), -CLAMP, CLAMP);
    return (unlean(y) - unlean(seen)) * SPREAD;
}

fn density(noise: f32, cover: f32) -> f32 {
    let start = threshold(cover);
    return smoothstep(start, start + EDGE, noise);
}

fn broad_cover(map: texture_cube<f32>, map_sampler: sampler, dir: vec3<f32>, toward_second: f32) -> f32 {
    let held = textureSampleLevel(map, map_sampler, dir, 0.0);
    return mix(held.r, held.g, toward_second);
}

fn puff(p: vec3<f32>, drift: vec4<f32>, sunward: vec3<f32>, facing: f32, cover: f32, footprint: f32, read: i32) -> Puff {
    let here = fbm_part(spot_at(p, drift), OCTAVES, read, footprint);
    let dense = density(here + graze(cover, facing), cover);
    var relief = RELIEF_TOP;
    if (read >= OCTAVES) {
        let toward_light = fbm_seen(spot_at(p + sunward * 0.3, drift), 2, footprint);
        relief = clamp(0.6 + (here - toward_light * 1.25) * 8.0, 0.0, 1.0);
    }
    return Puff(dense, relief);
}

fn lone(cloud: Puff, cover: f32) -> f32 {
    return mix(smoothstep(LONE.x, LONE.y, cloud.density), 1.0, clamp(cover, 0.0, 1.0));
}

fn overhead(p: vec3<f32>, drift: vec4<f32>, cover: f32, footprint: f32) -> f32 {
    let dense = density(fbm_seen(spot_at(p, drift), OCTAVES, footprint), cover);
    return dense * lone(Puff(dense, 0.0), cover);
}

fn day_side(color: vec3<f32>, n: vec3<f32>, light: vec3<f32>) -> vec3<f32> {
    return color * mix(NIGHT_SIDE, 1.0, smoothstep(-0.2, 0.25, dot(n, light)));
}

fn shade(base: vec3<f32>, cloud: Puff, above: bool) -> vec3<f32> {
    let body = select(mix(1.0, 0.72, cloud.density), mix(0.82, 1.05, cloud.density), above);
    return mix(base * 0.55, base * 1.05, cloud.relief) * body;
}
