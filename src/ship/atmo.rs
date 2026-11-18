//! Flight in the surface frame of a planet: gravity, the turning frame, air, hover and ground contact.

use bevy::math::{DQuat, DVec3};

use super::assist::{Hold, Hover, NEUTRAL, airless, lift};
use super::body::Ship;
use super::drag::{density, pressure, slow, stability};
use super::flight::{Controls, Report, main_engine, rotate};
use super::frames::spin_rate;
use super::landing::{Contact, contact};
use super::rest::{settle, wakes};
use crate::universe::PlanetSpec;
use crate::wind::thickness;

/// A planet and the terrain height in metres above its sea level at a position in its frame.
pub type Land<'a> = (&'a PlanetSpec, &'a dyn Fn(DVec3) -> f64);

/// Returns the acceleration of gravity and of the turning frame at `pos`, without the Coriolis part.
fn weight(planet: &PlanetSpec, pos: DVec3) -> DVec3 {
    let omega = spin_rate(planet);
    let outward = DVec3::new(pos.x, 0.0, pos.z) * (omega * omega);
    outward - pos * (planet.mu() / pos.length().powi(3))
}

/// Returns the acceleration of the engines in frame axes: the main engine and the vertical thrusters.
fn engines(ship: &Ship, controls: &Controls, touch: &Contact, down: DVec3) -> DVec3 {
    let up = ship.pos.normalize();
    let hover = Hover {
        stick: controls.lift,
        armed: ship.assist && ship.throttle.abs() < NEUTRAL,
        radar: touch.radar,
        climb: ship.vel.dot(up),
        weight: -down.dot(up),
        tilt: (ship.rot * DVec3::Y).dot(up),
        touching: touch.ground,
    };
    ship.rot * (main_engine(ship, controls) + DVec3::Y * lift(&hover))
}

/// Returns what the assist adds over `h` seconds where the air of `planet` is too thin to turn
/// and slow the ship: the rules it follows in space, against the ground instead of a body.
/// Nothing in home air or thicker, and nothing with the assist off.
fn held(ship: &Ship, controls: &Controls, planet: &PlanetSpec, h: f64) -> Option<DVec3> {
    let share = airless(thickness(&planet.record));
    let hold = Hold {
        drift: ship.vel,
        nose: ship.rot * DVec3::NEG_Z,
        thrusting: ship.throttle > NEUTRAL,
        braking: controls.brake,
    };
    (ship.assist && share > 0.0).then(|| hold.push(h) * share)
}

/// Returns the report of a ship at rest on the ground: it feels the ground push against its weight.
fn parked(ship: &Ship, planet: &PlanetSpec) -> Report {
    Report {
        proper: -weight(planet, ship.pos),
        radar: 0.0,
        ..Report::default()
    }
}

/// Advances a ship in the surface frame of a planet by `h` seconds.
///
/// Semi-implicit Euler: forces from the old position give the new velocity, which moves the ship.
/// The Coriolis force is applied as an exact turn of the velocity, so it cannot add energy.
pub fn step(ship: &mut Ship, controls: &Controls, land: Land, h: f64) -> Report {
    let (planet, ground) = land;
    if ship.landed && !wakes(ship, controls) {
        return parked(ship, planet);
    }
    ship.landed = false;
    let touch = contact(ship, planet, ground);
    let down = weight(planet, ship.pos);
    let driven = engines(ship, controls, &touch, down) + touch.accel;
    let pushed = held(ship, controls, planet, h).map_or(driven, |hold| driven + hold);
    let air = density(planet, ship.pos.length() - planet.radius);
    let coriolis = DQuat::from_rotation_y(-2.0 * spin_rate(planet) * h);
    let free = coriolis * (ship.vel + (down + pushed) * h);
    let torque = touch.torque + stability(ship.rot, free, ship.spin, air);
    ship.vel = slow(ship.rot, free, air, h);
    ship.pos += ship.vel * h;
    rotate(ship, controls.turn, torque, h);
    Report {
        proper: pushed + (ship.vel - free) / h,
        impact: settle(ship, &touch, controls, h),
        pressure: pressure(air, ship.vel.length()),
        radar: touch.radar,
    }
}
