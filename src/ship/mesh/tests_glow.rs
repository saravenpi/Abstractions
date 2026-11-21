//! Tests that the glow of an entry sits on the hull: the hot point of the sheath is the tip of
//! the nose, the bow shock stands off it by the 0.14 nose radii of a blunt body, nothing of the
//! sheath is centred ahead of the nose, and no lamp lights the hull from in front of it.

use super::frame::FUSELAGE;
use crate::ship::HULL_RADIUS;

/// The shader of the sheath, whose lengths are in radii of the hull.
const SHADER: &str = include_str!("../../render/heat/sheath.wesl");

/// The code of the sheath.
const SHEATH: &str = include_str!("../../render/heat/sheath.rs");

/// Radius in metres of the curve of the nose the heating is worked out for.
const NOSE_RADIUS: f32 = 0.5;

/// Returns the value of the `f32` constant `name` of the shader of the sheath.
fn constant(name: &str) -> f32 {
    let head = format!("const {name}: f32 = ");
    let line = SHADER
        .lines()
        .find(|line| line.starts_with(&head))
        .expect("the constant");
    line[head.len()..]
        .trim_end_matches(';')
        .parse()
        .expect("a number")
}

/// The nose of the sheath is the tip of the hull within 3 cm, the bow shock stands off it by at
/// most 0.14 nose radii and 2 cm, no glow is placed ahead of the nose, and no point light rides
/// in front of it.
#[test]
fn the_nose_glow_sits_on_the_nose() {
    let tip = -FUSELAGE[0][0];
    let nose = constant("NOSE") * HULL_RADIUS;
    assert!(
        (nose - tip).abs() < 0.03,
        "the glow sits at {nose} m, the tip at {tip} m"
    );
    let standoff = (constant("SHOCK_TIP") - constant("NOSE")) * HULL_RADIUS;
    assert!(
        standoff >= 0.0 && standoff < 0.14 * NOSE_RADIUS + 0.02,
        "{standoff} m"
    );
    assert!(
        !SHADER.contains("NOSE +"),
        "a glow is centred ahead of the nose"
    );
    assert!(
        !SHEATH.contains("PointLight"),
        "a lamp lights the hull from the shock"
    );
}
