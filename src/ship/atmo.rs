//! Flight in the surface frame of a planet or a moon: gravity, the turning frame, air, hover and
//! ground contact, or over a gas giant the floor and the deep air of `deck`.

use bevy::math::{DQuat, DVec3};

use super::assist::{NEUTRAL, TURN_RATE, airless, level};
use super::body::Ship;
use super::clear::guard;
use super::cruise::{AIR_GRIP, Cruise};
use super::deck::{self, grounded};
use super::drag::{density, pressure, slow, stability};
use super::flight::{Controls, Report, rotate, thrusters};
use super::frames::spin_rate;
use super::landing::Contact;
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

/// Metres per second a ship touches down at and rebounds at, at most, that the time it takes to come back
/// down is worked out from.
const REBOUND: f64 = 2.0;

/// Metres above the ground within which a ship that touched it is taken to be rebounding.
const REBOUND_HEIGHT: f64 = 4.0;

/// Returns the seconds after a touch of the gear in which a ship with the stick released is still being
/// set down, under the pull of `gravity` metres per second squared: the time a rebound at [`REBOUND`] takes
/// to rise and fall, between half a second and six. In the weak gravity of a moon a bounce is long, and an
/// assist that took hold of the ship in the air again would hop it up each time it touched.
fn bounce_time(gravity: f64) -> f64 {
    (2.0 * REBOUND / gravity.max(0.1)).clamp(0.5, 6.0)
}

/// Returns the acceleration of the engines in frame axes over a step of `h` seconds, where
/// the weight and the drag pull with `hold.0`, the assist keeps the share `hold.1` of its
/// push across the nose, the circular orbit at the ship runs at `hold.2` metres per second and
/// the air is `hold.3` kilograms per cubic metre.
///
/// Without assist the lever and the stick are thrusts. With it they ask for speeds against
/// the ground, and the weight and the drag are cancelled in full, so a ship that is let go
/// hangs still and a ship under way flies the speed asked for in any air. The lift of the deep
/// air of a giant is not in `hold.0`: the assist leaves it to float the ship back up. A ship
/// that touches the ground or the sea with idle engines is left to settle on it, unless the
/// stick asks to rise.
fn engines(
    ship: &Ship,
    controls: &Controls,
    touch: &Contact,
    hold: (DVec3, f64, f64, f64),
    h: f64,
) -> DVec3 {
    if !ship.assist {
        return thrusters(ship, controls);
    }
    let bounce = ship.since < bounce_time(hold.0.length()) && touch.radar < REBOUND_HEIGHT;
    let set_down = (touch.ground || touch.wet || bounce) && controls.lift <= 0.0;
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
        ground: touch.normal != DVec3::ZERO,
        grip: hold.1,
        orbital: hold.2,
        air: hold.3,
    };
    cruise.push(h) - hold.0
}

/// Height in metres of the gear above the ground and speed in metres per second under which
/// the assist also levels a ship under way once the stick is released, and the largest tilt
/// in radians it levels from there: a steeper climb or dive is kept, as Elite Dangerous does
/// within 30 degrees of the horizon (https://community.elitedangerous.com/en/node/348).
const AUTO_LEVEL: (f64, f64, f64) = (150.0, 30.0, std::f64::consts::FRAC_PI_6);

/// Share of the yaw rate of the attitude thrusters a landed or touching ship turns on the spot
/// at, on the yaw or the roll keys: 0.6 radians per second.
const TAXI: f64 = 0.85;

/// Returns true when the assist levels a ship whose stick is released: always with the
/// engines idle, and under way when it is near the ground, slow and within [`AUTO_LEVEL`].
fn levels(ship: &Ship, touch: &Contact) -> bool {
    let up = ship.pos.normalize();
    let tilt = (ship.rot * DVec3::Y).dot(up).clamp(-1.0, 1.0).acos();
    let near = touch.radar < AUTO_LEVEL.0 && ship.vel.length() < AUTO_LEVEL.1;
    ship.throttle < NEUTRAL || (near && tilt < AUTO_LEVEL.2)
}

/// Returns the stick the attitude thrusters get. Without assist it is the pilot's. With it a
/// ship on the ground only turns on the spot, a released stick levels the ship where
/// [`levels`] says so, and near the ground no part of the stick swings the hull into it.
fn stick(ship: &Ship, controls: &Controls, touch: &Contact) -> DVec3 {
    if !ship.assist {
        return controls.turn;
    }
    if touch.ground {
        return DVec3::Y * (controls.turn.y + controls.turn.z).clamp(-1.0, 1.0) * TAXI;
    }
    if controls.turn == DVec3::ZERO && levels(ship, touch) {
        return level(ship.rot, ship.pos.normalize());
    }
    guard(ship, touch, controls.turn)
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

/// Returns the report of a ship at rest on the ground, which the assist turns on the spot on
/// the yaw or roll keys for `h` seconds: it feels the ground push against its weight.
fn parked(ship: &mut Ship, controls: &Controls, planet: &PlanetSpec, h: f64) -> Report {
    if ship.assist {
        let rate = (controls.turn.y + controls.turn.z).clamp(-1.0, 1.0) * TAXI * TURN_RATE.y;
        ship.rot = (DQuat::from_axis_angle(ship.rot * DVec3::Y, rate * h) * ship.rot).normalize();
    }
    Report {
        proper: -weight(planet, ship.pos),
        radar: 0.0,
        ..Report::default()
    }
}

/// Books the touchdown of the step on the ship and returns the report of the step, given
/// `felt`, the acceleration the crew feels and the density of the air.
fn report(
    ship: &mut Ship,
    touch: &Contact,
    controls: &Controls,
    felt: (DVec3, f64),
    h: f64,
) -> Report {
    let impact = settle(ship, touch, controls, h);
    Report {
        proper: felt.0,
        impact,
        crash: (impact > 0.0 && touch.crash).then_some(touch.at),
        pressure: pressure(felt.1, ship.vel.length()),
        radar: touch.radar,
    }
}

/// Advances a ship in the surface frame of a planet by `h` seconds.
///
/// Semi-implicit Euler: forces from the old position give the new velocity, which moves the ship.
/// The Coriolis force is applied as an exact turn of the velocity, so it cannot add energy.
/// The hits of the step change the velocity and the spin at once, before anything else, and
/// set a ship that is already in the ground back on it.
pub fn step(ship: &mut Ship, controls: &Controls, land: Land, h: f64) -> Report {
    let (planet, ground) = land;
    if ship.landed && !wakes(ship, controls) && grounded(Some(planet)) {
        return parked(ship, controls, planet, h);
    }
    ship.landed = false;
    let touch = deck::meet(ship, planet, ground, h);
    let air = density(planet, ship.pos.length() - planet.radius);
    let pull = weight(planet, ship.pos);
    let down = deck::weigh(planet, pull, ship.pos, air);
    let grip = AIR_GRIP + (1.0 - AIR_GRIP) * airless(thickness(&planet.record));
    let circular = (planet.mu() / ship.pos.length()).sqrt();
    let lost = (slow(ship.rot, ship.vel, air, h) - ship.vel) / h;
    let hold = (pull + lost, grip, circular, air);
    let held = engines(ship, controls, &touch, hold, h);
    let pushed = deck::buoyed(planet, ship, held, [pull, lost, down - pull]) + touch.accel;
    let coriolis = DQuat::from_rotation_y(-2.0 * spin_rate(planet) * h);
    let free = coriolis * (ship.vel + (down + pushed) * h);
    let calm = air.min(density(planet, 0.0));
    let torque = touch.torque + stability(ship.rot, free, ship.spin, calm);
    ship.vel = slow(ship.rot, free, air, h);
    deck::floor(ship, planet, h);
    ship.pos += ship.vel * h;
    let turn = stick(ship, controls, &touch);
    rotate(ship, turn, torque, h);
    follow(ship, &touch, h);
    let felt = pushed + (down - pull) + (ship.vel - free) / h;
    report(ship, &touch, controls, (felt, air), h)
}
