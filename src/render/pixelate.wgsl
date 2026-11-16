struct Pixelate {
    pixel_size: f32,
    levels: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> settings: Pixelate;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = max(settings.pixel_size, 1.0);
    let block = floor(in.position.xy / size);
    let center = (block + vec2<f32>(0.5)) * size;
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let texel = vec2<i32>(clamp(center, vec2<f32>(0.0), dims - vec2<f32>(1.0)));
    let color = textureLoad(screen_texture, texel, 0);
    let perceptual = pow(clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
    let banded = floor(perceptual * settings.levels + vec3<f32>(0.5)) / settings.levels;
    return vec4<f32>(pow(banded, vec3<f32>(2.2)), color.a);
}
