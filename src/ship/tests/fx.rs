//! Flight effects on the real air: the glow of a fast descent, none in powered flight, and a
//! shake that stays small and smooth.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{above_pole, fly, planet};
use crate::render::{Drive, Sequence, entry_glow};
use crate::ship::flight::Controls;
use crate::ship::fx::{pace, warp_level};
use crate::ship::shake::{REACH, eased, roughness, shake};
use crate::ship::telemetry::WarpState;
use crate::ship::warp::{CRUISE_CAP, JUMP_CAP};
use crate::universe::{GLY, PlanetSpec};

/// Returns the strongest glow of a flight of `seconds` with the assist on that starts `height`
/// metres above the sea of `planet` at velocity `vel`, nose first, with the lever at `throttle`
/// and `controls`.
fn brightest(
    planet: &PlanetSpec,
    start: (f64, DVec3),
    power: (f64, Controls),
    seconds: f64,
) -> f32 {
    glow_of(planet, start, power, (seconds, true))
}

/// Returns the strongest glow of the same flight, of `run.0` seconds, with the assist on or
/// off as `run.1` says. With the assist on and the engines idle the ship stops in the air, so
/// only a ship without assist falls in from the top of the air.
fn glow_of(
    planet: &PlanetSpec,
    start: (f64, DVec3),
    power: (f64, Controls),
    run: (f64, bool),
) -> f32 {
    let (seconds, assist) = run;
    let mut ship = above_pole(planet, start.0);
    ship.vel = start.1;
    ship.rot = DQuat::from_rotation_arc(DVec3::NEG_Z, start.1.normalize_or(DVec3::NEG_Z));
    (ship.throttle, ship.assist) = (power.0, assist);
    let mut glow = 0.0f32;
    fly(
        &mut ship,
        &power.1,
        (planet, &|_| 0.0),
        seconds,
        |ship, report| {
            let sink = -ship.vel.dot(ship.pos.normalize());
            glow = glow.max(entry_glow(report.pressure as f32, sink as f32));
        },
    );
    glow
}

/// Returns a velocity of `speed` metres per second that sinks at the share `steep` of it.
fn sinking(speed: f64, steep: f64) -> DVec3 {
    DVec3::new((1.0 - steep * steep).sqrt(), -steep, 0.0) * speed
}

/// Under full power or full boost, flying level or in a shallow descent never glows, and
/// neither does a landing approach.
#[test]
fn powered_flight_and_approaches_never_glow() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let level = Controls {
        lift: 0.4,
        ..Controls::default()
    };
    let boosted = Controls {
        boost: true,
        ..level
    };
    for (steep, controls) in [(0.0, level), (0.1, level), (0.0, boosted), (0.1, boosted)] {
        let start = (1500.0, sinking(180.0, steep));
        let glow = brightest(&planet, start, (1.0, controls), 10.0);
        assert!(glow < 0.01, "powered flight at slope {steep} glows {glow}");
    }
    let approach = (200.0, DVec3::new(20.0, -6.0, 0.0));
    let glow = brightest(&planet, approach, (0.0, Controls::default()), 20.0);
    assert_eq!(glow, 0.0, "a landing approach glows");
}

/// Falling in from the top of the air without assist glows, at any speed around the orbital
/// one, and so does a steep dive under power with it.
#[test]
fn a_fast_descent_glows() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let orbital = (planet.gravity * planet.radius).sqrt();
    let entries = [
        (0.7, 0.5, 0.5),
        (0.7, 1.0, 0.9),
        (1.0, 0.7, 0.9),
        (1.5, 0.7, 0.9),
    ];
    for (speed, steep, least) in entries {
        let entry = (3900.0, sinking(orbital * speed, steep));
        let glow = glow_of(&planet, entry, (0.0, Controls::default()), (60.0, false));
        assert!(
            glow > least,
            "entry at {speed} orbital, slope {steep}: {glow}"
        );
    }
    let level = Controls {
        lift: 0.4,
        ..Controls::default()
    };
    let dive = brightest(&planet, (1500.0, sinking(180.0, 0.5)), (1.0, level), 10.0);
    assert!(dive > 0.9, "a steep dive under power glows only {dive}");
}

/// The warp effect follows the drive, and the shake is zero in calm air, grows with the clouds
/// and the pressure, never leaves its reach and never jumps between two frames.
#[test]
fn warp_level_and_shake_are_bounded_and_smooth() {
    assert_eq!(warp_level(WarpState::Off), 0.0);
    assert!(warp_level(WarpState::Spooling(1.0)) < 0.5);
    assert_eq!(warp_level(WarpState::Active(1.0e9)), 1.0);
    assert_eq!(roughness(0.0, 0.0), 0.0);
    assert!(roughness(1.0, 0.0) > 0.3 && roughness(0.0, 9.0e4) > 0.9);
    assert!(roughness(1.0, 9.0e9) <= 1.0);
    assert_eq!(shake(12.3, 0.0), (Vec3::ZERO, Quat::IDENTITY));
    let (dt, mut level, mut last) = (1.0 / 60.0, 0.0, shake(0.0, 0.0));
    for frame in 1..3600 {
        level = eased(level, 1.0, 3.0, dt);
        let now = shake(frame as f32 * dt, level);
        assert!(now.0.length() <= REACH.0 * 1.5 && now.1.angle_between(Quat::IDENTITY) < 0.01);
        assert!(
            now.0.distance(last.0) < 0.012,
            "the view jumped on frame {frame}"
        );
        assert!(now.1.angle_between(last.1) < 0.004);
        last = now;
    }
    assert!((level - 1.0).abs() < 1e-3);
    assert_eq!(eased(5e-4, 0.0, 3.0, dt), 0.0);
}

/// The sky draws the stars at the speed the drive really has: the share of its top speed the
/// instruments report, `warp::fraction`, read back by `Sequence::speed` once the sequence
/// cruises, gives the speed of the drive again, from the slowest warp to the fastest jump.
#[test]
fn the_sky_reads_the_speed_of_the_drive_back() {
    for speed in [2000.0, 1.0e6, 3.0e9, CRUISE_CAP, 40.0 * GLY, JUMP_CAP] {
        let drive = Drive {
            charge: 1.0,
            speed: pace(WarpState::Active(speed)),
        };
        let cruise = Sequence::default().advance(drive, 100.0);
        let shown = f64::from(cruise.speed()) * GLY;
        assert!((shown / speed - 1.0).abs() < 0.01, "{speed}: {shown}");
    }
}
