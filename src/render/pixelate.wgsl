struct Pixelate {
    levels: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> settings: Pixelate;

const IGN_X: f32 = 0.06711056;
const IGN_Y: f32 = 0.00583715;
const IGN_SCALE: f32 = 52.9829189;

const WHITE_DITHER: f32 = 0.8;

const WHITE_SOFT: f32 = 0.5;

fn dither(texel: vec2<i32>) -> f32 {
    let p = vec2<f32>(texel);
    return fract(IGN_SCALE * fract(p.x * IGN_X + p.y * IGN_Y));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = vec2<i32>(in.position.xy);
    let color = textureLoad(screen_texture, texel, 0);
    let perceptual = pow(clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
    let bright = max(perceptual.r, max(perceptual.g, perceptual.b));
    let strength = mix(1.0, WHITE_DITHER, bright);
    let offset = 0.5 + (dither(texel) - 0.5) * strength;
    let banded = floor(perceptual * settings.levels + vec3<f32>(offset)) / settings.levels;
    let shown = mix(banded, perceptual, WHITE_SOFT * bright);
    return vec4<f32>(pow(shown, vec3<f32>(2.2)), color.a);
}
