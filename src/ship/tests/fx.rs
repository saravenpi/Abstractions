//! Flight effects on the real air: the glow of a fast fall from orbit, none in powered flight
//! near the ground, and a shake that stays small and smooth.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::{above_pole, fly, planet};
use crate::render::{Drive, Gas, Sequence, entry};
use crate::ship::flight::Controls;
use crate::ship::fx::{pace, warp_level};
use crate::ship::shake::{eased, roughness, shake};
use crate::ship::telemetry::WarpState;
use crate::ship::warp::{CRUISE_CAP, GALAXY_CAP, JUMP_CAP};
use crate::speed::shake::REACH;
use crate::universe::{EARTH_RADIUS, LY, PlanetSpec};

/// Returns the strongest glow of a flight of `run.0` seconds over `planet`, with the assist on or
/// off as `run.1` says, that starts `start.0` metres above the sea at velocity `start.1`, nose
/// first, with the lever at `power.0` and the controls `power.1`. With the assist on and the
/// engines idle the ship stops in the air, so only a ship without assist falls in from the top
/// of the air.
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
            let speed = ship.vel.length().max(1.0);
            let air = 2.0 * report.pressure / (speed * speed);
            glow = glow.max(entry(air, speed, Gas::NitrogenOxygen).heat);
        },
    );
    glow
}

/// Returns a velocity of `speed` metres per second that sinks at the share `steep` of it.
fn sinking(speed: f64, steep: f64) -> DVec3 {
    DVec3::new((1.0 - steep * steep).sqrt(), -steep, 0.0) * speed
}

/// Under full power or full boost, flying level, in a shallow descent or in a steep dive near
/// the ground glows no more than mildly, and a landing approach never glows.
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
    let flights = [(0.0, level), (0.1, level), (0.0, boosted), (0.5, level)];
    for (steep, controls) in flights {
        let start = (1500.0, sinking(180.0, steep));
        let glow = glow_of(&planet, start, (1.0, controls), (10.0, true));
        assert!(glow < 0.25, "powered flight at slope {steep} glows {glow}");
    }
    let approach = (200.0, DVec3::new(20.0, -6.0, 0.0));
    let glow = glow_of(&planet, approach, (0.0, Controls::default()), (20.0, true));
    assert_eq!(glow, 0.0, "a landing approach glows");
}

/// Falling in from the top of the air of a planet of the size of the Earth without assist, at
/// the speed of a low orbit and at a range of slopes, glows nearly in full.
#[test]
fn a_fall_from_orbit_glows() {
    let planet = planet(EARTH_RADIUS, 1.0e5, 86_400.0);
    let orbital = (planet.gravity * planet.radius).sqrt();
    for steep in [0.03, 0.06, 0.1] {
        let start = (9.9e4, sinking(orbital, steep));
        let glow = glow_of(&planet, start, (0.0, Controls::default()), (120.0, false));
        assert!(
            glow > 0.8,
            "a fall from orbit at slope {steep} glows only {glow}"
        );
    }
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
    for speed in [
        2000.0,
        1.0e6,
        3.0e9,
        CRUISE_CAP,
        40.0 * LY,
        JUMP_CAP,
        GALAXY_CAP,
    ] {
        let drive = Drive {
            charge: 1.0,
            speed: pace(WarpState::Active(speed)),
        };
        let cruise = Sequence::default().advance(drive, 100.0);
        let shown = f64::from(cruise.speed()) * LY;
        assert!((shown / speed - 1.0).abs() < 0.01, "{speed}: {shown}");
    }
}

/// Climbing steeply out of an atmosphere keeps dynamic pressure while extinguishing re-entry glow.
#[test]
fn exiting_atmosphere_keeps_pressure_without_entry_glow() {
    let gas = Default::default();
    let entry = crate::ship::heat::glow_for(0.05, 4_000.0, -800.0, gas);
    let level = crate::ship::heat::glow_for(0.05, 4_000.0, 0.0, gas);
    let mid = crate::ship::heat::glow_for(0.05, 4_000.0, 600.0, gas);
    let climb = crate::ship::heat::glow_for(0.05, 4_000.0, 1_200.0, gas);
    assert!(entry.heat > 0.0 && entry.flux > 0.0 && entry.kelvin > 0.0 && entry.shine > 0.0);
    assert!(entry.pressure > 0.0);
    assert!((level.heat - entry.heat).abs() < 1e-5);
    assert!(mid.heat > 0.0 && mid.heat < entry.heat);
    assert!(mid.flux > 0.0 && mid.flux < entry.flux);
    assert!((mid.pressure - entry.pressure).abs() < 1e-5);
    assert_eq!(climb.heat, 0.0);
    assert_eq!(climb.flux, 0.0);
    assert_eq!(climb.kelvin, 0.0);
    assert_eq!(climb.shine, 0.0);
    assert!((climb.pressure - entry.pressure).abs() < 1e-5);
}

/// Re-entry sheath and vapour cone lock to the ship's propagated world pose after moving.
#[test]
fn sheath_and_cone_follow_post_update_pose() {
    use bevy::math::{DQuat, DVec3};
    use bevy::transform::TransformSystems;

    use crate::render::{Craft, Heat, HeatSet};
    use crate::ship::Ship;
    use crate::ship::exhaust::Exhaust;
    use crate::ship::wake::cone;
    use crate::space::{Frame, Location, Travel};
    use crate::universe::BodyId;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin));
    let moved = Vec3::new(420.0, 18.0, -75.0);
    let ship = app
        .world_mut()
        .spawn((
            Ship::at_rest(Frame::Surface(BodyId::default()), DVec3::Y, DQuat::IDENTITY),
            Exhaust::default(),
            Transform::from_translation(moved),
        ))
        .id();
    let mut heat = Heat::default();
    heat.offer(
        crate::ship::heat::glow_for(0.05, 4_000.0, -800.0, Default::default()),
        Craft {
            centre: Vec3::new(-900.0, 0.0, 0.0),
            heading: Vec3::NEG_Z,
            radius: 4.5,
            roof: Vec3::Y,
        },
    );
    heat.craft.centre = Vec3::new(-900.0, 0.0, 0.0);
    app.insert_resource(heat);
    app.insert_resource(Location {
        body: BodyId::default(),
        travel: Travel::InShip,
    });
    app.world_mut().spawn((
        cone::Vapour { ship, shown: 0.6 },
        Transform::from_translation(Vec3::new(-900.0, 0.0, 0.0)),
    ));
    let check = move |heat: Res<Heat>| {
        assert!((heat.craft.centre - moved).length() < 1e-4);
    };
    app.add_systems(
        PostUpdate,
        (
            crate::ship::heat::align
                .after(TransformSystems::Propagate)
                .before(HeatSet),
            check.in_set(HeatSet),
            cone::follow.after(TransformSystems::Propagate),
        ),
    );
    app.update();
    let aligned = app.world().resource::<Heat>();
    assert!(
        (aligned.craft.centre - moved).length() < 1e-4,
        "craft stayed at {:?}",
        aligned.craft.centre
    );
    let mut cones = app.world_mut().query::<(&cone::Vapour, &GlobalTransform)>();
    let (_, cone_pose) = cones.single(app.world()).expect("cone present");
    assert!(cone_pose.translation().distance(moved) < 20.0);
}
