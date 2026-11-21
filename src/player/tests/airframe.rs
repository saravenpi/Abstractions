//! The airframe of the landed ship as the body meets it: it stands on a wing and walks along
//! it, the winglet stops it, and past the wingtip it falls to the ground.

use std::f32::consts::FRAC_1_SQRT_2;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::climbing::{FLOOR, SOLES, put, soles, stage, walk};
use crate::player::BODY_RADIUS;
use crate::ship::airframe;

/// Where the ship root stands: its gear on the ground, its right wing pointing ahead of the
/// body, and the body on the chord of the wing 2.08 m behind the root.
const ROOT: Vec3 = Vec3::new(-2.08, FLOOR + 1.25, 0.0);

/// Chord position of the body on the wing: ship-root z. The swept wing ends in a tip 0.72 m
/// deep, from z 1.7 to 2.42. On this line the top of the wing is flat from 2.2 m out to the tip,
/// and the faces of the winglet are flat at the height of the body.
const CHORD: f32 = 2.08;

/// The turn of the ship root: a quarter turn about the vertical, so the right wing points along -z.
const TURN: Quat = Quat::from_xyzw(0.0, FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2);

/// Returns a stage at 30 frames a second with the landed ship on its ground.
fn landed() -> App {
    let mut app = stage(30.0);
    let root = Transform::from_translation(ROOT).with_rotation(TURN);
    app.world_mut()
        .spawn((RigidBody::Kinematic, airframe(), root));
    app
}

/// Returns the render position of the ship-root point `x` metres out along the right wing,
/// `y` metres up and on the chord the body walks.
fn at(x: f32, y: f32) -> Vec3 {
    ROOT + TURN * Vec3::new(x, y, CHORD)
}

/// Returns the ship-root position of the render point `world`.
fn local(world: Vec3) -> Vec3 {
    TURN.inverse() * (world - ROOT)
}

/// Returns the ship-root height of the top of the right wing `x` metres out: 0.04 m at its
/// root, 0.45 m out, falling to -0.16 m at its tip, 3.2 m out.
fn wing_top(x: f32) -> f32 {
    0.04 - 0.2 * (x - 0.45) / 2.75
}

/// A body dropped from 1 m above the outer panel of the wing lands on it and stays there, grounded.
#[test]
fn a_body_lands_on_the_wing() {
    let mut app = landed();
    put(&mut app, at(2.4, wing_top(2.4) + 1.0 + SOLES));
    let seen = walk(&mut app, 0.0, 60);
    for (transform, motion, _) in &seen[30..] {
        let body = local(transform.translation);
        assert!(motion.grounded, "{body}");
        assert!((body.y - SOLES - wing_top(2.4)).abs() < 0.05, "{body}");
        assert!((body.x - 2.4).abs() < 0.05, "{body}");
    }
}

/// A body walking out along the wing stays on it, grounded on its sloping top, and the winglet
/// stops it at the tip.
#[test]
fn a_body_walks_the_wing_to_the_winglet() {
    let mut app = landed();
    put(&mut app, at(2.2, wing_top(2.2) + SOLES + 0.02));
    walk(&mut app, 0.0, 10);
    let seen = walk(&mut app, 1.0, 60);
    for (transform, motion, _) in &seen {
        let body = local(transform.translation);
        assert!(motion.grounded, "{body}");
        assert!((body.y - SOLES - wing_top(body.x)).abs() < 0.06, "{body}");
        assert!(body.x < 3.15 - BODY_RADIUS + 0.02, "{body}");
    }
    let end = local(seen.last().expect("frames").0.translation);
    assert!(end.x > 3.15 - BODY_RADIUS - 0.1, "{end}");
}

/// A body walking on the ground into the side of the wingtip is stopped by the winglet: it
/// neither climbs it nor passes through it.
#[test]
fn the_winglet_stops_a_body_from_the_side() {
    let mut app = landed();
    put(&mut app, at(4.4, -1.25 + SOLES + 0.02));
    walk(&mut app, 0.0, 10);
    let seen = walk(&mut app, -1.0, 60);
    for (transform, ..) in &seen {
        let body = local(transform.translation);
        assert!(soles(transform) < 0.05, "{body}");
        assert!(body.x > 3.25 + BODY_RADIUS - 0.02, "{body}");
    }
    let end = local(seen.last().expect("frames").0.translation);
    assert!(end.x < 3.25 + BODY_RADIUS + 0.1, "{end}");
}

/// A body dropped beside the ship, past the wingtip, misses the wing and falls to the ground.
#[test]
fn a_body_past_the_wingtip_falls_to_the_ground() {
    let mut app = landed();
    put(&mut app, at(4.0, wing_top(3.2) + 1.0 + SOLES));
    let seen = walk(&mut app, 0.0, 60);
    let (transform, motion, _) = seen.last().copied().expect("frames");
    assert!(
        motion.grounded && soles(&transform).abs() < 0.05,
        "{}",
        transform.translation
    );
}
