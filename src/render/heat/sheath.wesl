import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::{view, globals};
import bevy_pbr::render::view_transformations::depth_ndc_to_view_z;
@if(DEPTH_PREPASS)
import bevy_pbr::prepass::utils::prepass_depth;

struct Sheath {
    centre: vec4<f32>,
    axis: vec4<f32>,
    roof: vec4<f32>,
    glow: vec4<f32>,
    skin: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sheath: Sheath;

struct Ray {
    o: vec3<f32>,
    d: vec3<f32>,
    near: f32,
    far: f32,
}

const ROOT_PI: f32 = 1.7724539;
const GAIN: f32 = 2.0;
const SHOCK_TIP: f32 = 0.972;
const SHOCK_OPEN: f32 = 2.82;
const SHOCK_THICK: f32 = 0.05;
const NOSE: f32 = 0.956;
const NOSE_ROOF: f32 = -0.037;
const WING_ROOT: vec3<f32> = vec3<f32>(0.289, 0.100, -0.020);
const WING_CRANK: vec3<f32> = vec3<f32>(0.022, 0.278, -0.037);
const WING_TIP: vec3<f32> = vec3<f32>(-0.378, 0.711, -0.046);
const TAIL: f32 = -0.75;
const FLOW: f32 = 3.0;
const CANOPY: f32 = 1.5;

fn tube(ray: Ray, a: vec3<f32>, b: vec3<f32>, width: vec2<f32>) -> vec2<f32> {
    let axis = b - a;
    let span = max(length(axis), 1.0e-4);
    let u = axis / span;
    let w = ray.o - a;
    let dd = dot(ray.d, u);
    let denom = max(1.0 - dd * dd, 1.0e-4);
    let along = (dot(u, w) - dd * dot(ray.d, w)) / denom;
    let s = clamp(along / span, 0.0, 1.0);
    let at = a + axis * s;
    let t = dot(at - ray.o, ray.d);
    let off = ray.o + ray.d * t - at;
    let sigma = mix(width.x, width.y, s);
    let seen = clamp((ray.far - t) / sigma + 0.5, 0.0, 1.0) * clamp((t - ray.near) / sigma + 0.5, 0.0, 1.0);
    let glow = ROOT_PI * sigma * exp(-dot(off, off) / (sigma * sigma)) / max(sqrt(denom), 0.25);
    return vec2<f32>(glow * seen, s);
}

fn sheet(ray: Ray, t: f32, time: f32) -> f32 {
    if (t <= ray.near || t >= ray.far) {
        return 0.0;
    }
    let p = ray.o + ray.d * t;
    let cross = p.yz - vec2<f32>(0.0, NOSE_ROOF);
    let across = dot(cross, cross);
    let normal = normalize(vec3<f32>(1.0, 2.0 * SHOCK_OPEN * cross.x, 2.0 * SHOCK_OPEN * cross.y));
    let facing = max(abs(dot(normal, ray.d)), 0.12);
    let strength = 0.35 + 1.4 * exp(-across / 0.08);
    let fade = smoothstep(-1.6, -0.5, p.x);
    let ripple = 0.85 + 0.15 * sin(atan2(cross.y, cross.x) * 5.0 + time * 3.0 + p.x * 4.0);
    return SHOCK_THICK * strength * fade * ripple / facing;
}

fn shock(ray: Ray, time: f32) -> f32 {
    let origin = ray.o.yz - vec2<f32>(0.0, NOSE_ROOF);
    let qa = SHOCK_OPEN * dot(ray.d.yz, ray.d.yz);
    let qb = ray.d.x + 2.0 * SHOCK_OPEN * dot(origin, ray.d.yz);
    let qc = ray.o.x - SHOCK_TIP + SHOCK_OPEN * dot(origin, origin);
    if (qa < 1.0e-5) {
        return sheet(ray, -qc / qb, time);
    }
    let disc = qb * qb - 4.0 * qa * qc;
    if (disc < 0.0) {
        return 0.0;
    }
    let root = sqrt(disc);
    return sheet(ray, (-qb - root) / (2.0 * qa), time) + sheet(ray, (-qb + root) / (2.0 * qa), time);
}

fn stagnation(ray: Ray) -> f32 {
    let tip = vec3<f32>(NOSE, 0.0, NOSE_ROOF);
    return 10.0 * tube(ray, tip, tip - vec3<f32>(0.09, 0.0, -0.006), vec2<f32>(0.012, 0.022)).x;
}

fn edges(ray: Ray) -> f32 {
    let mirror = vec3<f32>(1.0, -1.0, 1.0);
    let strake = vec2<f32>(0.035, 0.0315);
    let outer = vec2<f32>(0.0315, 0.025);
    let right = max(
        tube(ray, WING_ROOT, WING_CRANK, strake).x,
        tube(ray, WING_CRANK, WING_TIP, outer).x,
    );
    let left = max(
        tube(ray, WING_ROOT * mirror, WING_CRANK * mirror, strake).x,
        tube(ray, WING_CRANK * mirror, WING_TIP * mirror, outer).x,
    );
    return 2.0 * (right + left);
}

fn envelope(ray: Ray) -> f32 {
    let body = tube(ray, vec3<f32>(0.71, 0.0, 0.0), vec3<f32>(-0.53, 0.0, 0.0), vec2<f32>(0.127, 0.23));
    return 0.7 * body.x * mix(1.0, 0.5, body.y);
}

fn wake(ray: Ray, time: f32) -> f32 {
    let sway = vec3<f32>(0.0, sin(time * 1.3), cos(time * 1.1)) * 0.2;
    let trail = tube(ray, vec3<f32>(TAIL, 0.0, 0.0), vec3<f32>(TAIL - 8.0, 0.0, 0.0) + sway, vec2<f32>(0.25, 1.1));
    return 0.6 * trail.x * exp(-2.5 * trail.y);
}

fn streamers(ray: Ray, time: f32, rough: f32) -> f32 {
    var sources = array<vec3<f32>, 3>(
        WING_TIP,
        WING_TIP * vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(TAIL, 0.0, 0.12),
    );
    var sum = 0.0;
    for (var j = 0; j < 3; j++) {
        let seed = f32(j);
        let phase = fract(time * FLOW + seed * 0.37);
        for (var k = 0; k < 3; k++) {
            let age = f32(k) * 1.6 + phase;
            let wobble = vec3<f32>(0.0, sin(time * 2.3 + age * 1.7 + seed * 2.0), cos(time * 1.9 + age * 1.3 + seed));
            let head = sources[j] - vec3<f32>(age * 0.9, 0.0, 0.0) + wobble * (0.04 * age * (0.5 + rough));
            let streak = tube(ray, head, head - vec3<f32>(0.6, 0.0, 0.0), vec2<f32>(0.05, 0.08)).x;
            let life = smoothstep(0.0, 0.6, age) * (1.0 - age / 5.0);
            sum += streak * life * (0.7 + 0.3 * sin(time * 11.0 + seed * 2.1 + age));
        }
    }
    return sum;
}

fn scene_far(frag: vec4<f32>, travelled: f32) -> f32 {
    var far = 1.0e6;
    @if(DEPTH_PREPASS) {
        let raw = prepass_depth(frag, 0u);
        if (raw > 0.0) {
            far = travelled * depth_ndc_to_view_z(raw) / depth_ndc_to_view_z(frag.z);
        }
    }
    return far;
}

fn local_ray(in: VertexOutput) -> Ray {
    let radius = sheath.centre.w;
    let axis = normalize(sheath.axis.xyz);
    let pick = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), step(0.9, abs(axis.y)));
    let lifted = sheath.roof.xyz - axis * dot(sheath.roof.xyz, axis);
    let fallback = pick - axis * dot(pick, axis);
    let roof = normalize(select(fallback, lifted, dot(lifted, lifted) > 1.0e-6));
    let span = cross(roof, axis);
    let eye = view.world_position;
    let travelled = max(distance(in.world_position.xyz, eye), 1.0e-4);
    let dir = (in.world_position.xyz - eye) / travelled;
    let rel = (eye - sheath.centre.xyz) / radius;
    let o = vec3<f32>(dot(rel, axis), dot(rel, span), dot(rel, roof));
    let d = vec3<f32>(dot(dir, axis), dot(dir, span), dot(dir, roof));
    let near = select(0.0, CANOPY / radius, length(o) < 1.0);
    return Ray(o, d, near, scene_far(in.position, travelled) / radius);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray = local_ray(in);
    let time = globals.time;
    let gas = sheath.glow.rgb;
    let skin = sheath.skin.rgb;
    let core = mix(skin, vec3<f32>(1.0), 0.45);
    let cool = mix(gas, vec3<f32>(1.0, 0.42, 0.22), 0.35);
    let pulse = 1.0 + 0.1 * sin(time * 9.0) + 0.07 * sin(time * 23.0 + 1.3);
    let hot = (core * (1.5 * stagnation(ray)) + skin * (1.4 * edges(ray))) * sheath.skin.w;
    let shocked = gas * (shock(ray, time) + 0.6 * envelope(ray));
    let trailing = cool * (0.9 * wake(ray, time) + 0.8 * streamers(ray, time, sheath.roof.w));
    let plasma = (shocked + trailing) * (sheath.glow.w * pulse);
    let colour = (hot + plasma) * GAIN;
    return vec4<f32>(colour, 0.0);
}
