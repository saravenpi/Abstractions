//! Tests of the pure parts of the wind, rush, nozzle and particle effects.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::flare::pose;
use super::haze::scale;
use super::motes::{PUFF_HEIGHT, rates};
use super::scene::Scene;
use super::streaks::{SPEED, noise, strength};
use super::trail::Trail;
use super::trails::tip_heat;
use crate::gear::{FlightState, Mode};

/// Returns an active scene of a body in `mode` at `airspeed` under `load` g, in sea level air.
fn scene(mode: Mode, airspeed: f32, load: f32) -> Scene {
    let winged = f32::from(u8::from(mode.winged()));
    Scene {
        active: true,
        sigma: 1.0,
        clearance: 100.0,
        tilt: Quat::IDENTITY,
        state: FlightState {
            mode,
            airspeed,
            load,
            wings: winged,
            thrust: 1.0,
            velocity: Vec3::NEG_Z * airspeed,
            ..FlightState::default()
        },
        ..Scene::default()
    }
}

/// No speed lines below 15 m/s, more with speed, none in vacuum.
#[test]
fn speed_lines_scale_with_airspeed() {
    assert_eq!(strength(SPEED.0, 1.0), 0.0);
    assert_eq!(strength(8.0, 1.0), 0.0);
    assert!(strength(30.0, 1.0) > 0.0 && strength(30.0, 1.0) < strength(50.0, 1.0));
    assert_eq!(strength(80.0, 1.0), 1.0);
    assert_eq!(strength(80.0, 0.0), 0.0);
    assert!((0..64).all(|slot| (0.0..1.0).contains(&noise(slot, 3))));
}

/// The wingtips glow faintly in steady flight and brightly under load and in dives, and not
/// at all folded, slow or on the ground.
#[test]
fn wingtips_glow_under_load() {
    let steady = tip_heat(&scene(Mode::Glide, 45.0, 1.0));
    assert!(steady > 0.0 && steady < 0.5);
    assert!(tip_heat(&scene(Mode::Glide, 45.0, 2.5)) > 2.0 * steady);
    assert!(tip_heat(&scene(Mode::Glide, 90.0, 1.0)) > 1.5 * steady);
    assert_eq!(tip_heat(&scene(Mode::Glide, 12.0, 1.0)), 0.0);
    assert_eq!(tip_heat(&scene(Mode::Fall, 60.0, 1.0)), 0.0);
    assert_eq!(tip_heat(&scene(Mode::Slide, 45.0, 1.0)), 0.0);
}

/// The rush ring is out of sight at rest and inside the view at full rush, at any aspect.
#[test]
fn rush_ring_closes_in() {
    for aspect in [1.0, 16.0 / 9.0, 21.0 / 9.0] {
        let (idle, full) = (scale(1.2, aspect, 0.0), scale(1.2, aspect, 1.0));
        let half = (0.6_f32).tan() * 0.4;
        assert!(idle.y / half > std::f32::consts::SQRT_2, "visible at rest");
        assert!(full.y / half < 1.0 && full.x < idle.x);
        assert!((idle.x / idle.y - aspect).abs() < 1e-4);
    }
}

/// Sparks need a braked fall, puffs need a surface close below, and neither needs the other.
#[test]
fn boots_spark_when_braking_and_puff_near_a_surface() {
    let braking = scene(Mode::BootFall, 30.0, 2.4);
    let hovering = scene(Mode::BootFall, 5.0, 1.0);
    assert!(rates(&braking).0 > 20.0 && rates(&braking).1 == 0.0);
    assert_eq!(rates(&hovering), (0.0, 0.0));
    let low = Scene {
        clearance: 1.0,
        ..hovering
    };
    assert!(rates(&low).1 > 10.0);
    let edge = Scene {
        clearance: PUFF_HEIGHT,
        ..hovering
    };
    assert_eq!(rates(&edge).1, 0.0);
    assert_eq!(
        rates(&Scene {
            state: FlightState::default(),
            ..low
        }),
        (0.0, 0.0)
    );
}

/// A flare hangs under its own boot, grows with the push and vanishes without it.
#[test]
fn flares_grow_with_the_push() {
    let (left, right) = (pose(-1.0, 1.0, 0.3), pose(1.0, 1.0, 0.3));
    assert!(left.translation.x < 0.0 && right.translation.x > 0.0);
    assert!(left.translation.y < -0.9);
    assert!(pose(1.0, 0.3, 0.3).scale.y < right.scale.y);
    assert_eq!(pose(1.0, 0.0, 0.3).scale, Vec3::ZERO);
}

/// A long jump starts the trail over, and a trail nobody feeds heat into stops being live.
#[test]
fn trail_breaks_on_jumps_and_dies_out() {
    let mut trail = Trail::default();
    for step in 0..60 {
        trail.feed(DVec3::X * f64::from(step), 1.0, 1.0 / 60.0);
    }
    assert!(trail.len() > 30);
    trail.feed(DVec3::X * 9000.0, 1.0, 1.0 / 60.0);
    assert_eq!(trail.len(), 1);
    assert!(trail.live(1.5));
    for step in 0..120 {
        trail.feed(DVec3::X * (9000.0 + f64::from(step)), 0.0, 1.0 / 60.0);
    }
    assert!(!trail.live(1.5));
}
