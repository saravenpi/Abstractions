//! Tests of the shadow fade and of the light correction of `relit` in `daylight.wesl`, redone
//! here on the CPU, and of the rule that takes the occlusion pass off a high camera.

use bevy::prelude::*;

use super::occluding;
use super::tune::{OCCLUSION_CEILING, OCCLUSION_MARGIN, OCCLUSION_RANGE, SHADOW_FADE};
use crate::render::smoothstep;

/// The shader that holds the correction for the GPU.
const SHADER: &str = include_str!("../daylight.wesl");

/// Returns the numbers of the constant `name` of the shader, in the order they are written.
fn shader_constant(name: &str) -> Vec<f32> {
    let start = format!("const {name}:");
    let line = SHADER.lines().find(|line| line.starts_with(&start));
    let value = line.and_then(|line| line.split('=').nth(1)).unwrap_or("");
    let bare = value.replace("vec2<f32>", "");
    let numbers = bare.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'));
    numbers.filter_map(|number| number.parse().ok()).collect()
}

/// Returns the share of the shadows `sun_shadow` of the shader gives back at `depth` metres in
/// front of the camera when the last cascade ends `far` metres away.
fn fade(depth: f32, far: f32) -> f32 {
    smoothstep(far * (1.0 - SHADOW_FADE), far, depth)
}

/// Returns the light `relit` of the shader adds to the sun light of the scene at a point where
/// the rule gives `here` and the camera gets `eye`, in shadow `share` (1 in the open) and where
/// the shadows are given back by `fade`.
fn correction(here: Vec3, eye: Vec3, share: f32, fade: f32) -> Vec3 {
    (here - eye) * share + here * ((1.0 - share) * fade)
}

/// The shader holds the numbers of the Rust side under the same names.
#[test]
fn the_shader_holds_the_same_constants() {
    assert_eq!(shader_constant("SHADOW_FADE"), [SHADOW_FADE]);
    assert_eq!(
        shader_constant("OCCLUSION_RANGE"),
        [OCCLUSION_RANGE.0, OCCLUSION_RANGE.1]
    );
    for line in [
        "let back = (1.0 - shadow.share) * shadow.fade;",
        "let sun = (shift * shadow.share + here.sun * back) * sun_response(in, place.star.xyz);",
        "let shift = here.sun - place.eye_sun.rgb;",
    ] {
        assert!(SHADER.contains(line), "{line}");
    }
}

/// At the camera the rule gives what the scene light gives, and the correction is zero in
/// every shadow, in the open or not.
#[test]
fn the_correction_is_zero_at_the_camera() {
    let eye = Vec3::new(11_000.0, 10_500.0, 9_800.0);
    for share in [0.0, 0.3, 1.0] {
        assert_eq!(correction(eye, eye, share, 0.0), Vec3::ZERO, "{share}");
    }
}

/// Under full shadow the correction takes nothing from a shadow and never goes below zero, and
/// it does not light a shadow where the shadow maps still reach.
#[test]
fn the_correction_never_lights_or_darkens_a_shadow() {
    let eye = Vec3::new(11_000.0, 10_500.0, 9_800.0);
    for here in [Vec3::ZERO, eye * 0.2, eye, eye * 1.1] {
        let shadowed = correction(here, eye, 0.0, 0.0);
        assert_eq!(shadowed, Vec3::ZERO);
        for fade in [0.0, 0.5, 1.0] {
            assert!(correction(here, eye, 0.0, fade).min_element() >= 0.0);
        }
    }
}

/// Past the last cascade the shadows are all given back, and the light of the place is whole.
#[test]
fn the_shadows_end_in_a_fade() {
    let (here, eye) = (Vec3::splat(9_000.0), Vec3::splat(9_000.0));
    assert_eq!(correction(here, eye, 0.0, fade(110.0, 110.0)), here);
    let depths: Vec<f32> = (0..=120).map(|m| fade(m as f32, 110.0)).collect();
    assert_eq!(depths[0], 0.0);
    assert_eq!(fade(110.0 * (1.0 - SHADOW_FADE), 110.0), 0.0);
    assert!(depths.windows(2).all(|pair| pair[1] >= pair[0]));
    assert!(depths.windows(2).all(|pair| pair[1] - pair[0] < 0.2));
}

/// The occlusion pass stays on near the ground, goes off above the ceiling and comes back under
/// it, with a margin so that it does not flicker, and never runs when the setting is off.
#[test]
fn the_occlusion_pass_leaves_a_high_camera() {
    assert!(occluding(true, 1.7, false) && occluding(true, 1.7, true));
    assert!(!occluding(false, 1.7, true));
    let between = OCCLUSION_CEILING + OCCLUSION_MARGIN * 0.5;
    assert!(occluding(true, between, true) && !occluding(true, between, false));
    assert!(!occluding(true, OCCLUSION_CEILING + OCCLUSION_MARGIN, true));
    assert!(OCCLUSION_CEILING >= OCCLUSION_RANGE.1);
}
