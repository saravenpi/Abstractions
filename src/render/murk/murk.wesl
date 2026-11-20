import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::view::View;

struct Murk {
    up: vec3<f32>,
    height: f32,
    colour: vec3<f32>,
    density: f32,
    radius: f32,
    under: f32,
}

@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;
@group(0) @binding(2) var<uniform> murk: Murk;
@group(0) @binding(3) var<uniform> view: View;

const FAR: f32 = 1.0e9;
const LINE_SHADE: f32 = 0.35;
const LINE_PIXELS: f32 = 1.5;

fn world_at(uv: vec2<f32>, z: f32) -> vec3<f32> {
    let clip = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), z, 1.0);
    let world = view.world_from_clip * clip;
    return world.xyz / world.w;
}

fn height_of(d: vec3<f32>) -> f32 {
    let a = murk.radius + murk.height;
    let s = 2.0 * a * dot(murk.up, d) + dot(d, d);
    return murk.height + s / (sqrt(max(a * a + s, 0.0)) + a);
}

fn to_surface(h: f32, rise: f32) -> f32 {
    let a = murk.radius + h;
    let c = h * (2.0 * murk.radius + h);
    let q = sqrt(max(a * a * rise * rise - c, 0.0));
    if (rise > 0.0) {
        return -c / max(a * rise + q, 1.0e-6);
    }
    return q - a * rise;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let texel = vec2<i32>(in.position.xy);
    let color = textureLoad(screen, texel, 0);
    let z = textureLoad(depth, texel, 0);
    let lens = world_at(in.uv, 1.0) - view.world_position;
    let h = height_of(lens);
    let ray = normalize(lens);
    let start = length(lens);
    let hit = select(FAR, length(world_at(in.uv, z) - view.world_position), z > 0.0);
    let through = select(0.0, max(min(hit - start, to_surface(h, dot(ray, murk.up))), 0.0), h < 0.0);
    let fogged = mix(color.rgb, murk.colour, 1.0 - exp(-murk.density * through));
    let edge = max(fwidth(h) * LINE_PIXELS, 1.0e-5);
    let line = 1.0 - smoothstep(0.0, edge, abs(h));
    return vec4<f32>(mix(fogged, murk.colour * LINE_SHADE, line), color.a);
}
