import bevy_pbr::render::forward_io::VertexOutput;

struct Ring {
    color: vec4<f32>,
    light: vec4<f32>,
    shape: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> ring: Ring;

const NIGHT_SIDE: f32 = 0.04;
const TAU: f32 = 6.2831853;
const RINGLETS: f32 = 5.0;
const RING_DEPTH: f32 = 0.6;
const RING_EDGE: f32 = 0.04;
const IGN: vec3<f32> = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
const IGN_SHIFT: vec2<f32> = vec2<f32>(17.0, 31.0);

fn dither(texel: vec2<f32>) -> f32 {
    let p = texel + IGN_SHIFT;
    return fract(IGN.z * fract(p.x * IGN.x + p.y * IGN.y));
}

fn ring_cover(u: f32, inner: f32, density: f32) -> f32 {
    if (u < inner || u > 1.0) {
        return 0.0;
    }
    let t = (u - inner) / max(1.0 - inner, 0.0001);
    let edge = smoothstep(inner, inner + RING_EDGE, u) * (1.0 - smoothstep(1.0 - RING_EDGE, 1.0, u));
    let wave = TAU * RINGLETS * t + 1.7 * sin(TAU * 1.3 * t);
    return density * edge * (1.0 - RING_DEPTH * (0.5 + 0.5 * cos(wave)));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let cover = ring_cover(length(in.uv), ring.shape.x, ring.color.w);
    if (cover <= dither(in.position.xy)) {
        discard;
    }
    let p = vec3<f32>(in.uv.x, 0.0, in.uv.y);
    let l = ring.light.xyz;
    let along = dot(p, l);
    let miss = length(p - l * along) / ring.shape.y;
    let open = select(1.0, smoothstep(0.97, 1.03, miss), along < 0.0);
    let lit = mix(NIGHT_SIDE, 1.0, open);
    return vec4<f32>(ring.color.rgb * lit, 1.0);
}
