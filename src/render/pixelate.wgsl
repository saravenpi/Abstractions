struct Pixelate {
    pixel_size: u32,
    levels: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> settings: Pixelate;

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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = vec2<i32>(in.position.xy);
    let color = textureLoad(screen_texture, texel, 0);
    let perceptual = pow(clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
    let strength = mix(1.0, 0.4, max(perceptual.r, max(perceptual.g, perceptual.b)));
    let offset = 0.5 + (dither(texel) - 0.5) * strength;
    let banded = floor(perceptual * settings.levels + vec3<f32>(offset)) / settings.levels;
    return vec4<f32>(pow(banded, vec3<f32>(2.2)), color.a);
}
