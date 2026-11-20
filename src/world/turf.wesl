// The carpet of the ground-cover species: two colours in patches, a fine speckle, and how thick it
// grows, read from the second uv set of a turfed mesh. The ground and the blades share it, so a
// blade always takes the colour of the carpet it stands in.
//
// The second uv set holds the chart position of the vertex wrapped to PERIOD metres per chunk, its
// `y` lifted by BAND for each quarter of turf: level 0 is bare, level 4 fully covered.

struct Turf {
    // Linear RGB of the commonest ground-cover species; `a` is 1 on a planet with turf.
    low: vec4<f32>,
    // Linear RGB of the next one; `a` is how stiffly its blades stand, 0 to 1.
    high: vec4<f32>,
    // Height of a grown blade in metres, the density of the carpet from 0 to 1, then unused.
    shape: vec4<f32>,
}

const PERIOD: f32 = 256.0;
const BAND: f32 = 512.0;
const LEVELS: f32 = 4.0;

fn hash(c: vec2<i32>) -> f32 {
    var h = (u32(c.x) * 73856093u) ^ (u32(c.y) * 19349663u);
    h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
    h = h ^ (h >> 12u);
    return f32(h & 0xffffu) / 65535.0;
}

fn wrapped(c: vec2<i32>, n: i32) -> vec2<i32> {
    return ((c % n) + n) % n;
}

fn lattice(p: vec2<f32>, cell: f32) -> f32 {
    let n = i32(PERIOD / cell);
    let q = p / cell;
    let i = vec2<i32>(floor(q));
    let f = q - floor(q);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash(wrapped(i, n));
    let b = hash(wrapped(i + vec2<i32>(1, 0), n));
    let c = hash(wrapped(i + vec2<i32>(0, 1), n));
    let d = hash(wrapped(i + vec2<i32>(1, 1), n));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn level(uv_b: vec2<f32>) -> f32 {
    return floor(uv_b.y / BAND);
}

fn cover(turf: Turf, uv_b: vec2<f32>) -> f32 {
    return clamp(level(uv_b) / LEVELS, 0.0, 1.0) * turf.low.a;
}

fn spot(uv_b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(uv_b.x, uv_b.y - level(uv_b) * BAND);
}

fn speckle(p: vec2<f32>) -> f32 {
    return hash(wrapped(vec2<i32>(floor(p * 4.0)), i32(PERIOD * 4.0)));
}

fn carpet(turf: Turf, p: vec2<f32>) -> vec3<f32> {
    let patch = lattice(p, 8.0) * 0.65 + lattice(p, 2.0) * 0.35;
    let t = smoothstep(0.35, 0.65, patch);
    return mix(turf.low.rgb, turf.high.rgb, t) * (0.84 + 0.32 * speckle(p));
}
