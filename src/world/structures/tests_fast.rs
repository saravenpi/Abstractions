//! Tests that a fast body does not tunnel through a thin slab: the capsule of the player, moved the way its
//! controller moves it on foot and on the wings, stops on a 0.1 m slab at 80 m/s at every frame rate.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::touch::{HALF_BODY, pushed, stage};
use crate::player::BODY_RADIUS;

/// Thickness in metres of the slab, the thickness of the box under a roof slope and of the canopy of the base.
const SLAB: f32 = 0.1;

/// Speed in metres per second of the body, above the fastest dive on the wings.
const SPEED: f32 = 80.0;

/// Frames a second the body is moved at: the slowest step the controller takes, a usual one and a fast one.
const RATES: [f32; 3] = [20.0, 60.0, 144.0];

/// Returns a stage with a level slab whose top is at height zero and an upright one whose near face is at
/// `x = 0`, 30 m away from the first.
fn slabs() -> App {
    let level = Collider::cuboid(8.0, SLAB, 8.0);
    let upright = Collider::cuboid(SLAB, 8.0, 8.0);
    let parts = vec![
        (Vec3::new(30.0, -SLAB * 0.5, 0.0), Quat::IDENTITY, level),
        (Vec3::new(SLAB * 0.5, 0.0, 0.0), Quat::IDENTITY, upright),
    ];
    stage(Collider::compound(parts))
}

/// A body that falls on a thin level slab at 80 m/s ends standing on it, at every frame rate.
#[test]
fn a_fast_fall_stops_on_a_thin_slab() {
    let mut app = slabs();
    for rate in RATES {
        let from = Vec3::new(30.0, 40.0, 0.3);
        let end = pushed(&mut app, from, Vec3::NEG_Y * SPEED, (rate as u32, rate));
        let over = end.y - HALF_BODY;
        assert!(
            (0.0..0.1).contains(&over),
            "at {rate} frames a second the fall ended {over} m over the slab"
        );
    }
}

/// A body that flies level into a thin upright slab at 80 m/s ends against its near face, at every frame rate.
#[test]
fn a_fast_flight_stops_at_a_thin_wall() {
    let mut app = slabs();
    for rate in RATES {
        let from = Vec3::new(-40.0, 0.5, 0.3);
        let end = pushed(&mut app, from, Vec3::X * SPEED, (rate as u32, rate));
        let short = -end.x - BODY_RADIUS;
        assert!(
            (0.0..0.1).contains(&short),
            "at {rate} frames a second the flight ended {short} m in front of the wall"
        );
    }
}

/// A body that dives at a slant onto a thin level slab at 80 m/s stays above it and slides along it.
#[test]
fn a_fast_slanted_dive_stays_above_a_thin_slab() {
    let mut app = slabs();
    for rate in RATES {
        let from = Vec3::new(27.0, 10.0, -2.0);
        let dive = Vec3::new(0.3, -1.0, 0.2).normalize() * SPEED;
        let frames = (rate * 0.2) as u32;
        let end = pushed(&mut app, from, dive, (frames, rate));
        assert!(
            end.y - HALF_BODY > -0.01,
            "at {rate} frames a second the dive ended at {end}"
        );
    }
}
