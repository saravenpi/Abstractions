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

const WIND: vec3<f32> = vec3<f32>(0.011, 0.0, 0.0045);
const SPACE_COLOR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
const NIGHT_SIDE: f32 = 0.04;
const EDGE: f32 = 0.12;
const GRAZE: f32 = 0.08;
const LIMB: f32 = 0.3;
const IGN: vec3<f32> = vec3<f32>(0.06711056, 0.00583715, 52.9829189);
const IGN_SHIFT: vec2<f32> = vec2<f32>(17.0, 31.0);

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
    let here = fbm(spot, 4);
    let toward_light = fbm(spot + to_fixed * light * 0.3, 2);
    let to_eye = view.world_position - in.world_position.xyz;
    let dist = length(to_eye);
    let facing = dot(n, to_eye / max(dist, 0.001));
    let threshold = mix(0.72, 0.4, cover);
    let limb = LIMB * pow(1.0 - abs(facing), 4.0);
    let density = smoothstep(threshold, threshold + EDGE, here + limb);
    let thick = 1.0 - pow(1.0 - density, 1.0 / max(abs(facing), GRAZE));
    let far = smoothstep(0.35, 1.0, dist / max(clouds.params.z, 1.0));
    let alpha = max(thick * (1.0 - 0.75 * far * air), whiteout);
    if (alpha <= dither(in.position.xy)) {
        discard;
    }

    let relief = clamp(0.6 + (here - toward_light * 1.25) * 8.0, 0.0, 1.0);
    let body = select(mix(1.0, 0.72, density), mix(0.82, 1.05, density), facing > 0.0);
    let day = smoothstep(-0.2, 0.25, dot(n, light));
    let from_space = SPACE_COLOR * mix(NIGHT_SIDE, 1.0, day);
    let base = mix(from_space, clouds.color.rgb, air);
    var col = mix(base * 0.55, base * 1.05, relief) * body;
    col = mix(col, clouds.haze.rgb, far * air * 0.8);
    col = mix(col, clouds.color.rgb, whiteout);
    return vec4<f32>(col, 1.0);
}
