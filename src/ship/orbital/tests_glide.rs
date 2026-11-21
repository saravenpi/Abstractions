//! Tests of the glide from the cruise into the air: flown by the real flight step with the
//! assist, it ends low, a few kilometres short of the target with the target ahead, and the nose
//! never glows on the way.

use bevy::math::{DQuat, DVec3};

use super::glide::{self, Fix, handover, order};
use super::rig::earth;
use super::skim::{Band, attitude};
use super::tune::{GLIDE_ANGLE, GLIDE_STICK};
use crate::render::cool_speed;
use crate::ship::body::Ship;
use crate::ship::drag::density;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::steer::aim;
use crate::space::Frame;
use crate::universe::PlanetSpec;

/// Seconds the glide may take from the cruise floor to its end: about 72 s on an Earth, the time
/// the speed at which the nose glows takes to get down through the air.
const LIMIT: f64 = 85.0;

/// How a glide ended: the ship, the seconds it took, whether it ended, and the highest share of
/// the speed at which the nose glows that it flew at.
struct Flown {
    /// The ship at the end.
    ship: Ship,
    /// Seconds of the glide.
    time: f64,
    /// Whether the glide said it was over.
    done: bool,
    /// Highest speed over the speed at which the nose glows, on the way.
    worst: f64,
}

/// Returns a ship in the surface frame of `planet` where the cruise hands over to the glide
/// toward +X on the equator, on the glide path at the glide speed.
fn handed(planet: &PlanetSpec) -> Ship {
    let height = Band::of(planet).floor;
    let start = DQuat::from_rotation_y(handover(height) / planet.radius) * DVec3::X;
    let toward = (DVec3::X - start * start.x).normalize();
    let path = toward * GLIDE_ANGLE.cos() - start * GLIDE_ANGLE.sin();
    let rot = attitude(start, toward, GLIDE_ANGLE);
    let pos = start * (planet.radius + height);
    let mut ship = Ship::at_rest(Frame::Surface(planet.id), pos, rot);
    ship.vel = path * glide::speed(planet, height);
    ship
}

/// Flies one step of the glide toward +X over `planet` in `scene` and returns whether the glide
/// says it is over.
fn fly(ship: &mut Ship, planet: &PlanetSpec, scene: &Scene) -> bool {
    let fix = Fix {
        pos: ship.pos,
        frame: DVec3::ZERO,
        goal: DVec3::X,
        ground: 0.0,
    };
    let asked = order(&fix, planet);
    (ship.throttle, ship.assist) = (asked.throttle, true);
    let controls = Controls {
        turn: aim(ship.rot, asked.aim).clamp_length_max(GLIDE_STICK),
        boost: true,
        ..Controls::default()
    };
    step(ship, &controls, scene, STEP);
    asked.done
}

/// Flies the glide to +X over a sea-level `planet` with the flight step until it is over or
/// [`LIMIT`] seconds have passed.
fn glide_down(planet: &PlanetSpec) -> Flown {
    let scene = Scene {
        planet: Some(planet),
        ground: &|_| 0.0,
        wells: &[],
        elapsed: 0.0,
    };
    let mut ship = handed(planet);
    let (mut time, mut done, mut worst) = (0.0, false, 0.0f64);
    while time < LIMIT && !done {
        done = fly(&mut ship, planet, &scene);
        let height = ship.pos.length() - planet.radius;
        worst = worst.max(ship.vel.length() / cool_speed(density(planet, height)));
        time += STEP;
    }
    Flown {
        ship,
        time,
        done,
        worst,
    }
}

/// From the cruise floor over an Earth the glide ends within [`LIMIT`] seconds, 1 to 8 km above
/// the ground and under 15 km short of the target, flying toward it, and the nose never glows.
#[test]
fn the_glide_ends_low_and_short_of_the_target_with_the_nose_dark() {
    let planet = earth();
    let flown = glide_down(&planet);
    assert!(flown.done, "the glide ran {} s", flown.time);
    let height = flown.ship.pos.length() - planet.radius;
    assert!((1000.0..8000.0).contains(&height), "ended {height} m up");
    let up = flown.ship.pos.normalize();
    let arc = up.angle_between(DVec3::X) * planet.radius;
    assert!(arc < 15_000.0, "ended {arc} m from the target");
    let toward = (DVec3::X - up * up.x).normalize();
    assert!(flown.ship.vel.dot(toward) > 0.0, "the target is behind");
    assert!(
        flown.worst < 1.0,
        "the nose glowed: {} of the bound",
        flown.worst
    );
}

/// Through the air the glide never asks for a speed at which the nose would glow.
#[test]
fn the_glide_asks_for_a_dark_nose_at_every_height() {
    let planet = earth();
    for km in 0..140 {
        let height = f64::from(km) * 1000.0;
        let asked = glide::speed(&planet, height);
        let cool = cool_speed(density(&planet, height));
        assert!(
            asked > 0.0 && asked < cool,
            "{asked} m/s at {km} km, over {cool}"
        );
    }
}
