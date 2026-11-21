//! Tests of the pure parts of the wingtip, nozzle and particle effects, and of what lies
//! under the boots.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::flare::pose;
use super::motes::{PUFF_HEIGHT, rates};
use super::scene::Scene;
use super::trail::Trail;
use super::trails::tip_heat;
use super::under::under;
use crate::gear::{FlightState, Mode};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Globe;

/// Returns an active scene of a body in `mode` at `airspeed` under `load` g, in sea level air.
pub(super) fn scene(mode: Mode, airspeed: f32, load: f32) -> Scene {
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

/// The wingtips glow plainly in steady flight and brighter under load and in dives, and not
/// at all folded, slow or on the ground.
#[test]
fn wingtips_glow_under_load() {
    let steady = tip_heat(&scene(Mode::Glide, 45.0, 1.0));
    assert!(steady > 0.0 && steady < 0.7);
    assert!(tip_heat(&scene(Mode::Glide, 45.0, 2.5)) > 1.5 * steady);
    assert!(tip_heat(&scene(Mode::Glide, 90.0, 1.0)) > 1.5 * steady);
    assert_eq!(tip_heat(&scene(Mode::Glide, 12.0, 1.0)), 0.0);
    assert_eq!(tip_heat(&scene(Mode::Fall, 60.0, 1.0)), 0.0);
    assert_eq!(tip_heat(&scene(Mode::Slide, 45.0, 1.0)), 0.0);
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

/// Over a sea the nearer of ground and water counts, as it always did; a planet without a
/// sea has only ground, even in a basin below the datum, so the boots raise dust there.
#[test]
fn a_dry_basin_raises_dust_not_spray() {
    let home = Globe::new(7, 6000.0, 9.8);
    assert_eq!(under(&home, 5.0, 2.0), (2.0, true));
    assert_eq!(under(&home, 1.0, 40.0), (1.0, false));
    assert_eq!(under(&home, 3.0, -0.5), (0.0, true));
    let record = PlanetRecord {
        class: PlanetClass::Desert,
        ocean: 0.0,
        ..PlanetRecord::TEMPERATE
    };
    let dry = home.clone().with_record(record);
    assert!(!dry.has_sea());
    assert_eq!(under(&dry, 1.0, -30.0), (1.0, false));
    assert_eq!(under(&dry, 5.0, 2.0), (5.0, false));
    assert_eq!(under(&dry, f32::INFINITY, -4.0), (f32::INFINITY, false));
}
