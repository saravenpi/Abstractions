// Backdrop of a menu: the low-resolution game image behind the node, blurred and darkened.
//
// The node covers the screen, so its UV is the screen UV. The source image is already small,
// so thirteen taps on two rings give a strong blur for the cost of thirteen samples per pixel
// of the menu, and nothing when no menu is open.

import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Backdrop {
    // x: share of the light removed, y: blur radius as a fraction of the screen height.
    params: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> backdrop: Backdrop;
@group(1) @binding(1)
var world: texture_2d<f32>;
@group(1) @binding(2)
var world_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(world));
    let reach = vec2<f32>(backdrop.params.y * size.y / size.x, backdrop.params.y);
    var sum = textureSample(world, world_sampler, in.uv).rgb;
    for (var i = 0; i < 6; i++) {
        let turn = 1.0471976 * f32(i);
        let near = vec2<f32>(cos(turn), sin(turn)) * 0.5;
        let far = vec2<f32>(cos(turn + 0.5235988), sin(turn + 0.5235988));
        sum += textureSampleLevel(world, world_sampler, in.uv + near * reach, 0.0).rgb;
        sum += textureSampleLevel(world, world_sampler, in.uv + far * reach, 0.0).rgb;
    }
    let blurred = sum / 13.0;
    return vec4<f32>(blurred * (1.0 - backdrop.params.x), 1.0);
}
