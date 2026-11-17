//! Drive tests: the warp lock, the drop out, the carried velocity and the rotation assist.

use bevy::math::{DQuat, DVec3};

use crate::ship::body::Ship;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::steer::{aim, detour};
use crate::ship::warp::{CRUISE_CAP, SPOOL_TIME, Warp, advance, clearance, request};
use crate::ship::wells::{PLANET_LOCK, Well, system_wells};
use crate::space::Frame;
use crate::universe::{SystemSpec, Universe};

/// Returns the home system and a ship at rest relative to its home planet, `radii` planet radii
/// from its centre on the side away from the star, nose toward the planet.
fn near_home(radii: f64) -> (SystemSpec, usize, Ship) {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let index = usize::from(universe.home().index);
    let planet = &system.planets[index];
    let well = Well::planet(planet, 0.0);
    let out = well.pos.normalize();
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, -out);
    let pos = well.pos + out * (planet.radius * radii);
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), pos, rot);
    ship.vel = well.vel + DVec3::new(30.0, -12.0, 55.0);
    (system, index, ship)
}

/// The drive refuses to spool inside 3 radii of a planet and spools outside.
#[test]
fn warp_refuses_inside_the_lock_sphere() {
    let (system, _, mut ship) = near_home(PLANET_LOCK - 0.05);
    let wells = system_wells(&system, 0.0);
    assert!(!request(&mut ship, &wells));
    assert_eq!(ship.warp, Warp::Off);
    let (_, _, mut ship) = near_home(PLANET_LOCK + 0.05);
    assert!(request(&mut ship, &wells));
    assert_eq!(ship.warp, Warp::Spooling { left: SPOOL_TIME });
    ship.pos = wells[0].pos + DVec3::X * wells[0].lock * 0.9;
    advance(&mut ship, &wells, CRUISE_CAP, 0.1);
    assert_eq!(ship.warp, Warp::Off);
}

/// A ship warping straight at a moving planet from the void drops out outside the lock sphere with
/// the velocity it had relative to the planet when it engaged.
#[test]
fn warp_drops_out_at_the_lock_distance() {
    let (system, index, mut ship) = near_home(4000.0);
    assert!(request(&mut ship, &system_wells(&system, 0.0)));
    let (mut time, mut fastest, dt) = (0.0, 0.0f64, 1.0 / 30.0);
    let mut entry = DVec3::ZERO;
    while ship.warp != Warp::Off && time < 600.0 {
        let wells = system_wells(&system, time);
        let home = wells[index + 1];
        ship.rot = DQuat::from_rotation_arc(DVec3::NEG_Z, (home.pos - ship.pos).normalize());
        if matches!(ship.warp, Warp::Spooling { .. }) {
            entry = ship.vel - home.vel;
        }
        advance(&mut ship, &wells, CRUISE_CAP, dt);
        time += dt;
        assert!(
            clearance(&system_wells(&system, time), ship.pos) > 0.0,
            "inside a lock at {time}"
        );
        if let Warp::Active { speed, .. } = ship.warp {
            fastest = fastest.max(speed);
        }
    }
    let home = Well::planet(&system.planets[index], time - dt);
    let gap = home.pos.distance(ship.pos) / home.radius;
    assert!(
        ship.warp == Warp::Off && time > SPOOL_TIME,
        "still in warp after {time} s"
    );
    assert!(
        gap > PLANET_LOCK && gap < PLANET_LOCK * 1.2,
        "dropped at {gap} radii"
    );
    assert!(fastest > 1e6, "top speed {fastest}");
    assert!((ship.vel - home.vel - entry).length() < 1e-6);
}

/// Far from every body the drive reaches interstellar speed.
#[test]
fn warp_is_fast_in_the_void() {
    let mut ship = Ship::at_rest(
        Frame::Galaxy(Default::default()),
        DVec3::ZERO,
        DQuat::IDENTITY,
    );
    assert!(request(&mut ship, &[]));
    advance(&mut ship, &[], CRUISE_CAP, SPOOL_TIME + 0.1);
    advance(&mut ship, &[], CRUISE_CAP, 1.0);
    assert_eq!(
        ship.warp,
        Warp::Active {
            carried: DVec3::ZERO,
            speed: CRUISE_CAP
        }
    );
    assert!((ship.pos - DVec3::NEG_Z * CRUISE_CAP).length() < 1.0);
}

/// The assist stops a tumbling ship within two seconds of releasing the stick, and only the assist does.
#[test]
fn assist_kills_rotation() {
    let scene = Scene {
        planet: None,
        ground: &|_| 0.0,
        wells: &[],
        elapsed: 0.0,
    };
    let mut ship = Ship::at_rest(
        Frame::System(Default::default()),
        DVec3::ZERO,
        DQuat::IDENTITY,
    );
    ship.spin = DVec3::new(1.2, -0.8, 1.5);
    let mut free = ship.clone();
    free.assist = false;
    for _ in 0..(2.0 / STEP) as usize {
        step(&mut ship, &Controls::default(), &scene, STEP);
        step(&mut free, &Controls::default(), &scene, STEP);
    }
    assert!(
        ship.spin.length() < 0.01,
        "still turning at {}",
        ship.spin.length()
    );
    assert_eq!(free.spin, DVec3::new(1.2, -0.8, 1.5));
    assert_eq!(free.vel, DVec3::ZERO);
}

/// The autopilot stick turns the nose onto any course and holds it there.
#[test]
fn aim_turns_the_nose_onto_the_course() {
    let scene = Scene {
        planet: None,
        ground: &|_| 0.0,
        wells: &[],
        elapsed: 0.0,
    };
    for course in [
        DVec3::Z,
        DVec3::X,
        DVec3::new(-0.4, 0.8, -0.2).normalize(),
        DVec3::NEG_Y,
    ] {
        let mut ship = Ship::at_rest(
            Frame::System(Default::default()),
            DVec3::ZERO,
            DQuat::IDENTITY,
        );
        for _ in 0..(8.0 / STEP) as usize {
            let controls = Controls {
                turn: aim(ship.rot, course),
                ..Controls::default()
            };
            step(&mut ship, &controls, &scene, STEP);
        }
        assert!(
            ship.forward().dot(course) > 0.9999,
            "{course}: {}",
            ship.forward()
        );
        assert!(ship.spin.length() < 0.01, "{course}: still turning");
    }
}

/// A course through the lock sphere of a body is bent beside it, and a clear course is kept.
#[test]
fn detour_goes_around_a_body_in_the_way() {
    let well = Well {
        pos: DVec3::ZERO,
        vel: DVec3::ZERO,
        mu: 1.0,
        radius: 1000.0,
        lock: 3000.0,
    };
    let from = DVec3::new(-30_000.0, 200.0, 0.0);
    let bent = detour(&[well], from, DVec3::X, 60_000.0);
    let closest = (well.pos - from).cross(bent).length();
    assert!(
        bent.dot(DVec3::X) > 0.9 && closest > well.lock * 1.5,
        "passes at {closest}"
    );
    assert_eq!(detour(&[well], from, DVec3::X, 20_000.0), DVec3::X);
    assert_eq!(
        detour(&[well], from + DVec3::Y * 9000.0, DVec3::X, 60_000.0),
        DVec3::X
    );
    assert_eq!(detour(&[well], -from, DVec3::X, 60_000.0), DVec3::X);
}
