//! The hull against what is built: each step the hull is swept along the path it flew, stops a
//! skin short of a house, a stall or the base, and slides the rest of the way along it. Its
//! approach then counts as a touch like the ground: a slow one is a bump, a fast one a strike
//! with the damage and the crash of [`strike`](super::strike).

use bevy::math::{DQuat, DVec3};

use super::body::Ship;
use super::flight::Report;
use super::landing::SPRINGS;
use super::rest::harm;
use super::strike::{CRASH, Motion};

/// Metres the hull keeps between itself and what it meets, so that the next sweep starts clear.
pub const SKIN: f64 = 0.03;

/// Most surfaces one step slides along: a wall, the floor under it and a corner.
const ROUNDS: usize = 3;

/// Most metres a hull found in a solid is pushed out of it in one step: a solid that streams in
/// around the ship lets it out smoothly, not in a jump.
const PUSH: f64 = 0.25;

/// Shortest move in metres worth a sweep.
const SHORTEST: f64 = 1e-4;

/// What a sweep of the hull met first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// Metres the hull travelled along the sweep before it touched.
    pub distance: f64,
    /// Unit normal of the surface it touched, out of the solid, in frame axes.
    pub normal: DVec3,
    /// Where it touched the solid, in the frame.
    pub point: DVec3,
    /// Metres the hull already was in the solid along `normal` where the sweep started: a hull
    /// turned into a solid is not swept into it, so it is pushed back out.
    pub depth: f64,
}

/// Sweeps the hull turned to the given orientation from a position along a move, all in frame
/// axes, and returns what it meets within the move and a skin beyond.
pub type Cast<'a> = &'a dyn Fn(DVec3, DQuat, DVec3) -> Option<Hit>;

/// Returns `v` without the part of it that goes into a surface of unit normal `n`.
fn flatten(v: DVec3, n: DVec3) -> DVec3 {
    v - n * v.dot(n).min(0.0)
}

/// Stops the point of the hull that met the solid of `hit` with the ship centred at `centre`:
/// the impulse that takes its approach along the normal acts at that point, so a hull caught
/// by an edge away from its centre turns over it instead of hanging there. Returns the
/// approach speed of that point in metres per second.
fn stop(ship: &mut Ship, centre: DVec3, hit: &Hit) -> f64 {
    let arm = ship.rot.inverse() * (hit.point - centre);
    let mut motion = Motion::of(ship);
    let into = -motion.at(ship, arm).dot(hit.normal);
    if into > 0.0 {
        let kick = hit.normal * (into / Motion::give(ship, arm, hit.normal));
        motion.push(ship, arm, kick);
    }
    (ship.vel, ship.spin) = (motion.vel, motion.spin);
    into
}

/// Moves the ship from `from` along the path the step took it to where it is now, as far as
/// `cast` lets the hull go, sliding along each surface it meets and stopping the point that
/// met it, and out of a solid it was turned into. Returns the fastest approach in metres per
/// second of a point that met a surface, or nothing when the path was clear.
pub fn sweep(ship: &mut Ship, from: DVec3, cast: Cast) -> Option<f64> {
    let (mut at, mut rest, mut closing) = (from, ship.pos - from, None);
    for _ in 0..ROUNDS {
        let length = rest.length();
        if length < SHORTEST {
            break;
        }
        let Some(hit) = cast(at, ship.rot, rest) else {
            at += rest;
            break;
        };
        let go = (hit.distance - SKIN).clamp(0.0, length) / length;
        at += rest * go + hit.normal * hit.depth.min(PUSH);
        rest = flatten(rest * (1.0 - go), hit.normal);
        let into = stop(ship, at, &hit);
        closing = Some(closing.unwrap_or(0.0f64).max(into));
    }
    if closing.is_some() {
        ship.pos = at;
    }
    closing
}

/// Books on the ship and on the `report` of its step a hull that met a solid at `closing`
/// metres per second, when `was` tells that it already touched one on the step before. A hit
/// above the safe speed of the hull damages it, and from the crash speed of the hull it is a
/// crash. A first touch is a touchdown of its speed; a scrape that goes on is none.
pub fn book(ship: &mut Ship, closing: Option<f64>, was: bool, report: &mut Report) {
    ship.scraped = closing.is_some();
    let Some(closing) = closing else {
        return;
    };
    let safe = SPRINGS[1].2;
    harm(ship, closing - safe);
    if was && closing <= safe {
        return;
    }
    report.impact = report.impact.max(closing);
    if closing >= CRASH[1] {
        report.crash = Some(ship.pos);
    }
}

/// Sweeps the ship along the step it just flew from `from` with `cast`, then books what the hull
/// met on it and on the `report` of the step.
pub fn meet(ship: &mut Ship, from: DVec3, cast: Cast, report: &mut Report) {
    let was = ship.scraped;
    let closing = sweep(ship, from, cast);
    book(ship, closing, was, report);
}
