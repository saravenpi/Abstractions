// The waves of a wake, shared by the water and the foam. A point is given in the frame of one
// packet: `s` metres along the way the source went and `c` metres across it. The rows are those
// of `Packet::rows`: `p` is the clock it was laid at, the speed its ring grows at, the wave number
// of the ring and its height; `q` is the wave number of the transverse waves, their height, the
// speed of the source and its size; `e` is the speed of the crests of the ring and its half width.
// `field.rs` holds the same formulas on the CPU.

struct SeaWakes {
    count: vec4<f32>,
    rows: array<vec4<f32>, 128>,
}

struct FoamWakes {
    count: vec4<f32>,
    rows: array<vec4<f32>, 192>,
}

const WAKE_LIFE: f32 = 3.0;
const WAKE_MOVING: f32 = 2.0;
const WAKE_ARC: f32 = 0.45;
const WAKE_SPAN: f32 = 0.03;
const WAKE_KEEP_START: f32 = 0.15;
const WAKE_KEEP_END: f32 = 0.5;
const WAKE_TAU: f32 = 6.2831853;
const WAKE_HALF_PI: f32 = 1.5707964;

fn wake_keep(k: f32, foot: f32) -> f32 {
    return 1.0 - smoothstep(WAKE_KEEP_START, WAKE_KEEP_END, foot * k / WAKE_TAU);
}

fn wake_arc(p: vec4<f32>, q: vec4<f32>, s: f32, r: f32) -> f32 {
    if (q.z <= WAKE_MOVING) {
        return 1.0;
    }
    let rear = acos(clamp(-s / r, -1.0, 1.0));
    let meet = WAKE_HALF_PI + asin(clamp(p.y / q.z, 0.0, 1.0));
    let miss = (rear - meet) / WAKE_ARC;
    return exp(-miss * miss);
}

fn wake_ring(p: vec4<f32>, q: vec4<f32>, e: vec4<f32>, at: vec2<f32>, age: f32, foot: f32) -> vec3<f32> {
    let r = length(at);
    let front = p.y * age;
    let off = r - front;
    if (r < 0.001 || abs(off) > 3.0 * e.y) {
        return vec3<f32>(0.0);
    }
    let spread = inverseSqrt(1.0 + front / max(q.w, 1.0));
    let envelope = exp(-(off / e.y) * (off / e.y)) * spread;
    let height = p.w * envelope * wake_arc(p, q, at.x, r) * wake_keep(p.z, foot);
    let phase = p.z * (r - e.x * age);
    let slope = -height * p.z * sin(phase) / r;
    return vec3<f32>(height * cos(phase), slope * at.x, slope * at.y);
}

fn wake_transverse(p: vec4<f32>, q: vec4<f32>, at: vec2<f32>, age: f32, foot: f32) -> vec3<f32> {
    if (q.y <= 0.0) {
        return vec3<f32>(0.0);
    }
    let behind = q.z * age - at.x;
    let along = at.x - 0.5 * q.z * age;
    let span = WAKE_SPAN * q.z;
    if (abs(along) > 3.0 * span) {
        return vec3<f32>(0.0);
    }
    let size = max(q.w, 1.0);
    let reach = max(behind, 0.0) * p.y / q.z + 0.5 * q.w;
    let across = at.y / (0.6 * reach);
    let window = exp(-(along / span) * (along / span)) * exp(-across * across);
    let thin = smoothstep(0.0, size, behind) / (1.0 + max(behind, 0.0) / (8.0 * size));
    let height = q.y * window * thin * wake_keep(q.x, foot);
    let phase = q.x * (at.x - q.z * age);
    return vec3<f32>(height * cos(phase), -height * q.x * sin(phase), 0.0);
}

fn wake_bit(p: vec4<f32>, q: vec4<f32>, e: vec4<f32>, at: vec2<f32>, now: f32, foot: f32) -> vec3<f32> {
    let age = now - p.x;
    if (age < 0.0 || age >= WAKE_LIFE) {
        return vec3<f32>(0.0);
    }
    let fade = (1.0 - age / WAKE_LIFE) * (1.0 - age / WAKE_LIFE);
    return (wake_ring(p, q, e, at, age, foot) + wake_transverse(p, q, at, age, foot)) * fade;
}
