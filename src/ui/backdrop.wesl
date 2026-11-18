// Backdrop of a menu: the low-resolution game image behind the node, blurred and darkened.
//
// The image is read at the place of the pixel on the screen, so a node of any size and place
// shows the part of the game that lies behind it. The source image is already small,
// so a disc of taps spread by the golden angle gives a wide blur for the cost of a few dozen
// samples per pixel of the menu, and nothing when no menu is open. The disc is turned by a
// different angle on every pixel, which trades the ghost copies of a fixed pattern for a fine
// grain, so the game reads as frosted glass and never as a picture.

import bevy_render::view::View;
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Backdrop {
    // x: share of the light removed, y: blur radius as a fraction of the screen height.
    params: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> view: View;
@group(1) @binding(0)
var<uniform> backdrop: Backdrop;
@group(1) @binding(1)
var world: texture_2d<f32>;
@group(1) @binding(2)
var world_sampler: sampler;

/// Number of taps on the disc.
const TAPS: i32 = 40;

/// The golden angle in radians, which spreads the taps evenly over the disc.
const GOLDEN: f32 = 2.3999632;

/// Returns a number from 0 to 1 that changes from one pixel to the next without a visible pattern.
fn grain(pixel: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(world));
    let reach = vec2<f32>(backdrop.params.y * size.y / size.x, backdrop.params.y);
    let screen = in.position.xy / view.viewport.zw;
    let start = 6.2831853 * grain(in.position.xy);
    var sum = vec3<f32>(0.0);
    for (var i = 0; i < TAPS; i++) {
        let far = sqrt((f32(i) + 0.5) / f32(TAPS));
        let turn = start + GOLDEN * f32(i);
        let at = screen + vec2<f32>(cos(turn), sin(turn)) * far * reach;
        sum += textureSampleLevel(world, world_sampler, at, 0.0).rgb;
    }
    let blurred = sum / f32(TAPS);
    return vec4<f32>(blurred * (1.0 - backdrop.params.x), 1.0);
}
