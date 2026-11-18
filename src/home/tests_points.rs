//! Tests of the named points of the base: where the robot wakes, the hatch and the pad.

use bevy::math::DVec3;

use super::Base;
use super::tests::base;
use super::tune::{DRY, FOOTPRINT};
use crate::player::{BODY_LENGTH, BODY_RADIUS, EYE_OFFSET};

/// Wing span and length in metres of the ship, read from its mesh in `src/ship/mesh.rs`.
const SHIP: (f32, f32) = (6.4, 7.9);

/// Returns the horizontal distance in metres between two points near the base.
fn apart(base: &Base, a: DVec3, b: DVec3) -> f64 {
    let d = a - b;
    (d - base.up * d.dot(base.up)).length()
}

/// The wake point, the pad and the hatch lie on dry ground inside the footprint, the hatch above
/// the ground and the wake point on it.
#[test]
fn named_points_are_on_dry_ground() {
    for seed in 1..=12 {
        let (base, globe) = base(seed);
        let above = |p: DVec3| p.length() - globe.radius() - f64::from(globe.height(p.normalize()));
        for point in [base.wake, base.pad, base.door] {
            assert!(
                base.covers(point, 0.0),
                "seed {seed}: a point leaves the footprint"
            );
            let ground = globe.height(point.normalize());
            assert!(ground > DRY - 0.5, "seed {seed}: a point is on wet ground");
        }
        let hatch = above(base.door);
        assert!(
            (1.2..3.5).contains(&hatch),
            "seed {seed}: hatch {hatch} m above ground"
        );
        let wake = above(base.wake);
        assert!(
            wake.abs() < 0.3,
            "seed {seed}: wake point {wake} m off the ground"
        );
        assert!(
            above(base.pad).abs() < 0.3,
            "seed {seed}: pad off the ground"
        );
    }
}

/// The wake point, the pad and the hatch are distinct and at the distances of the layout: the
/// robot wakes 7 to 8.5 m from the hatch, and the pad is 35 to 60 m from it.
#[test]
fn named_points_are_distinct_and_spaced() {
    for seed in 1..=12 {
        let (base, _) = base(seed);
        let hatch = apart(&base, base.wake, base.door);
        assert!(
            (7.0..8.5).contains(&hatch),
            "seed {seed}: wake {hatch} m from the hatch"
        );
        let walk = apart(&base, base.pad, base.door);
        assert!(
            (35.0..60.0).contains(&walk),
            "seed {seed}: pad {walk} m from the hatch"
        );
        assert!(apart(&base, base.wake, base.pad) > 35.0);
    }
}

/// Returns the height in metres of the eyes of the robot over its feet.
fn eyes() -> f64 {
    f64::from(BODY_LENGTH * 0.5 + BODY_RADIUS + EYE_OFFSET)
}

/// The robot wakes facing the hatch, seen from its eyes, with the pad off to one side, in view.
#[test]
fn robot_wakes_facing_the_hatch_with_the_ship_in_view() {
    let (base, _) = base(1);
    let look = base.look();
    assert!(look.dot(base.wake.normalize()).abs() < 1e-9);
    let eye = base.wake + base.wake.normalize() * eyes();
    assert!(look.dot((base.door - eye).normalize()) > 0.95);
    let ship = (base.pad - base.wake).normalize();
    let angle = look.dot(ship).acos().to_degrees();
    assert!(
        (15.0..45.0).contains(&angle),
        "ship {angle} degrees off the view"
    );
}

/// The pad pose stands a ship upright with its nose along the heading, on a pad that is larger
/// than the ship with three metres to spare all round.
#[test]
fn pad_takes_the_ship_with_a_margin() {
    let (base, _) = base(1);
    let (rot, up) = (base.pad_rot(), base.pad.normalize());
    assert!((rot * DVec3::Y).dot(up) > 1.0 - 1e-12);
    assert!((rot * DVec3::NEG_Z).dot(base.heading) > 1.0 - 1e-5);
    assert!(base.heading.dot(base.up).abs() < 1e-9);
    assert!(super::PAD_HALF * 2.0 >= SHIP.0.max(SHIP.1) + 6.0);
    let side = up.cross(base.heading);
    let reach = f64::from(super::PAD_HALF) - 0.1;
    assert!(base.on_pad(base.pad + up * 2.0));
    assert!(base.on_pad(base.pad + base.heading * reach + side * reach));
    assert!(!base.on_pad(base.pad + base.heading * (reach + 0.3)));
    assert!(!base.on_pad(base.wake));
}

/// The footprint is a disc around the centre, grown by the margin asked for.
#[test]
fn footprint_is_a_disc() {
    let (base, _) = base(1);
    assert!(base.covers(base.centre + base.east * (FOOTPRINT - 0.1), 0.0));
    assert!(!base.covers(base.centre + base.east * (FOOTPRINT + 0.1), 0.0));
    assert!(base.covers(base.centre + base.north * (FOOTPRINT + 5.0), 6.0));
    assert!(base.covers(base.centre + base.up * 50.0, 0.0));
}
