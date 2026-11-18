//! Flight in the surface frame of a planet: gravity, the turning frame, air, hover and ground contact.

use bevy::math::{DQuat, DVec3};

use super::assist::{NEUTRAL, airless, level};
use super::body::Ship;
use super::cruise::{AIR_GRIP, Cruise};
use super::drag::{density, pressure, slow, stability};
use super::flight::{Controls, Report, rotate, thrusters};
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

/// Returns the acceleration of the engines in frame axes over a step of `h` seconds, where
/// the weight and the drag pull with `hold.0` and the assist keeps the share `hold.1` of its
/// push across the nose.
///
/// Without assist the lever and the stick are thrusts. With it they ask for speeds against
/// the ground, and the weight and the drag are cancelled in full, so a ship that is let go
/// hangs still and a ship under way flies the speed asked for in any air. A ship that touches
/// the ground or the sea with idle engines is left to settle on it, unless the stick asks to
/// rise.
fn engines(ship: &Ship, controls: &Controls, touch: &Contact, hold: (DVec3, f64), h: f64) -> DVec3 {
    if !ship.assist {
        return thrusters(ship, controls);
    }
    let set_down = (touch.ground || touch.wet) && controls.lift <= 0.0;
    if set_down && ship.throttle < NEUTRAL {
        return DVec3::ZERO;
    }
    let up = ship.pos.normalize();
    let cruise = Cruise {
        vel: ship.vel,
        nose: ship.forward(),
        rise: up,
        up,
        throttle: ship.throttle,
        lift: controls.lift,
        boost: controls.boost,
        brake: controls.brake,
        clearance: touch.radar,
        grip: hold.1,
    };
    cruise.push(h) - hold.0
}

/// Returns the stick the attitude thrusters get: the one of the pilot, or with the assist on,
/// the engines idle and the stick released in the air, the one that levels the ship.
fn stick(ship: &Ship, controls: &Controls, touch: &Contact) -> DVec3 {
    let idle = ship.throttle < NEUTRAL && controls.turn == DVec3::ZERO;
    if ship.assist && idle && !touch.ground {
        level(ship.rot, ship.pos.normalize())
    } else {
        controls.turn
    }
}

/// Turns an assisted ship in the air and its path with the local vertical over the `h` seconds
/// it just flew, so that a level nose stays level and keeps its height all the way around a
/// small planet instead of flying off along the tangent.
fn follow(ship: &mut Ship, touch: &Contact, h: f64) {
    if !ship.assist || touch.ground {
        return;
    }
    let turn = DQuat::from_scaled_axis(ship.pos.cross(ship.vel) / ship.pos.length_squared() * h);
    ship.vel = turn * ship.vel;
    ship.rot = (turn * ship.rot).normalize();
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
    let grip = AIR_GRIP + (1.0 - AIR_GRIP) * airless(thickness(&planet.record));
    let air = density(planet, ship.pos.length() - planet.radius);
    let lost = (slow(ship.rot, ship.vel, air, h) - ship.vel) / h;
    let pushed = engines(ship, controls, &touch, (down + lost, grip), h) + touch.accel;
    let coriolis = DQuat::from_rotation_y(-2.0 * spin_rate(planet) * h);
    let free = coriolis * (ship.vel + (down + pushed) * h);
    let torque = touch.torque + stability(ship.rot, free, ship.spin, air);
    ship.vel = slow(ship.rot, free, air, h);
    ship.pos += ship.vel * h;
    let turn = stick(ship, controls, &touch);
    rotate(ship, turn, torque, h);
    follow(ship, &touch, h);
    Report {
        proper: pushed + (ship.vel - free) / h,
        impact: settle(ship, &touch, controls, h),
        pressure: pressure(air, ship.vel.length()),
        radar: touch.radar,
    }
}
