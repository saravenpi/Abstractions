//! Tests of the warm band a low star lights along the horizon on its side: the dome and the haze
//! of the ground draw it by the same function with the same numbers, it is gone once the star is
//! high, and it is brightest on the horizon under the star, fading round the horizon and up the sky.

use bevy::prelude::*;

use super::celestial::smoothstep;

/// The dome shader.
const SKY: &str = include_str!("sky.wesl");

/// The ground shader, whose haze takes the band.
const GROUND: &str = include_str!("daylight.wesl");

/// Returns the number of the `f32` constant `name` of `shader`, or NaN when it has none.
fn constant(shader: &str, name: &str) -> f32 {
    let start = format!("const {name}: f32 = ");
    let line = shader.lines().find(|line| line.starts_with(&start));
    let value = line.and_then(|line| line.strip_prefix(&start));
    value
        .and_then(|value| value.trim_end_matches(';').parse().ok())
        .unwrap_or(f32::NAN)
}

/// Returns the text of the function `sunset_band` of `shader`, up to its closing brace.
fn body(shader: &str) -> &str {
    let start = shader.find("fn sunset_band(").unwrap_or(0);
    let end = shader[start..]
        .find("\n}\n")
        .map_or(shader.len(), |end| start + end);
    &shader[start..end]
}

/// Returns the strength of the band for a ray `v` from a camera whose way up is `up` under a star
/// at `sun`, the ray `over` the horizon by that sine: `sunset_band` of the shaders, on the CPU.
fn band(v: Vec3, sun: Vec3, up: Vec3, over: f32) -> f32 {
    let get = |name: &str| constant(SKY, name);
    let rise = sun.dot(up);
    let (flat_v, flat_sun) = (v - up * v.dot(up), sun - up * rise);
    if flat_v.length().min(flat_sun.length()) < 1.0e-4 {
        return 0.0;
    }
    let across = 1.0 - flat_v.normalize().dot(flat_sun.normalize());
    let low = 1.0 - smoothstep(0.0, get("SUNSET_HIGH"), rise);
    let fall = (-across * get("SUNSET_SPREAD")).exp() * (-over.max(0.0) * get("SUNSET_RISE")).exp();
    get("SUNSET_GAIN") * low * fall
}

/// Returns the unit vector at the sine `height` over the horizon toward the azimuth `turn` in
/// radians from the x axis, with y up.
fn toward(height: f32, turn: f32) -> Vec3 {
    let flat = (1.0 - height * height).max(0.0).sqrt();
    Vec3::new(flat * turn.cos(), height, flat * turn.sin())
}

/// Both shaders hold the same numbers and the same function, so the haze at the horizon meets the
/// dome there with no seam.
#[test]
fn the_dome_and_the_haze_share_the_band() {
    for name in ["SUNSET_SPREAD", "SUNSET_RISE", "SUNSET_GAIN", "SUNSET_HIGH"] {
        let value = constant(SKY, name);
        assert!(value.is_finite() && value > 0.0, "{name}");
        assert_eq!(constant(GROUND, name), value, "{name}");
    }
    assert!(body(SKY).len() > 100);
    assert_eq!(body(SKY), body(GROUND));
    assert!(SKY.contains("sunset_band(v, sun_dir, sky.up.xyz, over_limb) * sky.sun.w"));
    assert!(GROUND.contains("sunset_band(toward, place.star.xyz, up, dot(toward, up) + dip)"));
}

/// A high star lights no band. A star on the horizon lights it at full strength on the horizon
/// under it, less round the horizon, less up the sky, and least on the far side.
#[test]
fn the_band_hugs_the_horizon_under_a_low_star() {
    let up = Vec3::Y;
    let high = constant(SKY, "SUNSET_HIGH");
    assert_eq!(band(toward(0.0, 0.0), toward(high, 0.0), up, 0.0), 0.0);
    let sun = toward(0.0, 0.0);
    let under = band(toward(0.0, 0.0), sun, up, 0.0);
    assert!((under - constant(SKY, "SUNSET_GAIN")).abs() < 1.0e-5);
    let round = [0.0, 0.5, 1.0, 2.0, 3.1].map(|turn| band(toward(0.0, turn), sun, up, 0.0));
    assert!(round.windows(2).all(|pair| pair[1] < pair[0]), "{round:?}");
    assert!(round[4] < 0.01 * under);
    let rising = [0.0, 0.1, 0.3, 0.6].map(|h| band(toward(h, 0.0), sun, up, h));
    assert!(
        rising.windows(2).all(|pair| pair[1] < pair[0]),
        "{rising:?}"
    );
    assert!(band(toward(0.0, 0.0), toward(0.1, 0.0), up, 0.0) < under);
    assert_eq!(band(up, sun, up, 1.0), 0.0);
}
