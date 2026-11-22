//! Tests that the glow of an entry sits on the hull: the hot point of the sheath is the tip of
//! the nose, the bow shock stands off it by the 0.14 nose radii of a blunt body, nothing of the
//! sheath is centred ahead of the nose, and no lamp lights the hull from in front of it.

use bevy::prelude::*;

use super::frame::FUSELAGE;
use super::wing::{STATIONS, top};
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
    let roof = constant("NOSE_ROOF") * HULL_RADIUS;
    assert!(
        (roof - FUSELAGE[0][3]).abs() < 0.01,
        "the glow sits at roof {roof} m, the tip at {} m",
        FUSELAGE[0][3]
    );
    let standoff = (constant("SHOCK_TIP") - constant("NOSE")) * HULL_RADIUS;
    assert!(
        (0.0..0.14 * NOSE_RADIUS + 0.02).contains(&standoff),
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

/// Returns the light a ray from `o` along the unit `d` gathers from a glowing tube from `a` to
/// `b` whose width runs from `width.0` to `width.1`: the `tube` of the shader of the sheath.
fn tube(o: Vec3, d: Vec3, (a, b): (Vec3, Vec3), width: (f32, f32)) -> f32 {
    let axis = b - a;
    let span = axis.length();
    let u = axis / span;
    let w = o - a;
    let dd = d.dot(u);
    let denom = (1.0 - dd * dd).max(1.0e-4);
    let s = ((u.dot(w) - dd * d.dot(w)) / denom / span).clamp(0.0, 1.0);
    let at = a + axis * s;
    let off = o + d * (at - o).dot(d) - at;
    let sigma = width.0 + (width.1 - width.0) * s;
    1.772_454 * sigma * (-off.length_squared() / (sigma * sigma)).exp() / denom.sqrt().max(0.25)
}

/// Returns the vector constant `name` of the shader of the sheath.
fn point(name: &str) -> Vec3 {
    let head = format!("const {name}: vec3<f32> = vec3<f32>(");
    let line = SHADER
        .lines()
        .find(|line| line.starts_with(&head))
        .expect("the constant");
    let numbers: Vec<f32> = line[head.len()..]
        .trim_end_matches(");")
        .split(',')
        .map(|n| n.trim().parse().expect("a number"))
        .collect();
    Vec3::new(numbers[0], numbers[1], numbers[2])
}

/// The heat of the edges is one glowing line along each leading edge, not a row of spots: no
/// spot is drawn anywhere, and seen square on from above, the glow along the span has no gap
/// and changes by under five percent between neighbours a hundredth of the span apart.
#[test]
fn the_edge_glow_runs_along_the_span() {
    assert!(!SHADER.contains("blob("), "the sheath draws spots");
    assert!(!SHADER.contains("WING_ROOT + lead"));
    assert!(SHADER.contains("tube(ray, WING_ROOT, WING_CRANK"));
    assert!(SHADER.contains("tube(ray, WING_CRANK, WING_TIP"));
    let (root, crank, tip) = (point("WING_ROOT"), point("WING_CRANK"), point("WING_TIP"));
    for (name, got, s) in [
        ("WING_ROOT", root, STATIONS[0]),
        ("WING_CRANK", crank, STATIONS[1]),
        ("WING_TIP", tip, STATIONS[3]),
    ] {
        let hull = Vec3::new(-s[1], s[0], top(s[0]) - 0.5 * s[3]);
        assert!(
            (got * HULL_RADIUS).distance(hull) < 0.02,
            "{name}: {:?} vs {:?}",
            got * HULL_RADIUS,
            hull
        );
    }
    let seen = |share: f32| {
        let at = if share <= 0.35 {
            root.lerp(crank, share / 0.35)
        } else {
            crank.lerp(tip, (share - 0.35) / 0.65)
        };
        let eye = at + Vec3::Z * 3.0;
        let inner = tube(eye, Vec3::NEG_Z, (root, crank), (0.035, 0.0315));
        let outer = tube(eye, Vec3::NEG_Z, (crank, tip), (0.0315, 0.025));
        inner.max(outer)
    };
    let along: Vec<f32> = (0..=100).map(|k| seen(k as f32 / 100.0)).collect();
    assert!(along.iter().all(|glow| *glow > 0.03), "{along:?}");
    for pair in along.windows(2) {
        assert!((pair[1] / pair[0] - 1.0).abs() < 0.05, "{pair:?}");
    }
}
