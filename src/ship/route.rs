//! Flying a route: steering onto the course, asking for the warp drive and ending on arrival.

mod align;
#[cfg(test)]
mod align_tests;
mod ascent;
#[cfg(test)]
mod ascent_tests;
mod block;
#[cfg(test)]
mod block_tests;
mod done;
#[cfg(test)]
mod done_tests;
mod goal;

#[cfg(test)]
pub(super) use align::{led, spool};
use ascent::{climb, climbs};
use block::{aligned, blocker};
pub(super) use done::finish;
pub(super) use goal::ARRIVAL;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::flight::Controls;
use super::helm::Helm;
use super::pilot::{Autopilot, Goal, Phase, Settle};
use super::sim::Cosmos;
use super::sky::Skies;
use super::steer::{aim, waypoint};
use super::warp::{CRUISE_CAP, JUMP_CAP, Warp};
use super::wells::Well;
use crate::computer::{Destination, Engage, RouteStatus, Target, ship_position};
use crate::space::UniPos;
use crate::universe::{GLY, Universe};

/// Distance in metres beyond which a route is a long jump: faster, and blind to the stars on the way.
const JUMP_DISTANCE: f64 = 50.0 * GLY;

/// Returns what the autopilot does next, and whether it asks to switch the warp drive.
fn next(ship: &Ship, blocked: bool, aligned: bool) -> (Phase, bool) {
    match ship.warp {
        Warp::Active { .. } => (Phase::Cruising, false),
        Warp::Spooling { .. } => (Phase::Spooling, false),
        Warp::Off if blocked => (Phase::Waiting, false),
        Warp::Off => (
            if aligned {
                Phase::Spooling
            } else {
                Phase::Aligning
            },
            aligned,
        ),
    }
}

/// Returns the destination `to` metres from `pos` as a body with a lock sphere and no pull.
fn beacon(pos: DVec3, to: DVec3, lock: f64) -> Well {
    Well {
        pos: pos + to,
        vel: DVec3::ZERO,
        mu: 0.0,
        radius: lock,
        lock,
        hill: lock,
    }
}

/// Turns the ship onto the course to a destination `to` metres away with a lock sphere of
/// `lock` metres, asks for the drive once the nose is on the course and still, and returns what
/// the autopilot is doing.
///
/// Once the drive spools or runs the heading is locked: no stick, no spin. A body in the way is
/// not steered round in warp: the leg flies to a point beside it and the drive drops out there,
/// and the next leg starts from it. A course that the heading no longer reaches drops the drive
/// out too, and the ship aligns again before it warps. While the drive is locked the pilot
/// keeps the controls.
pub(super) fn hold(
    ship: &mut Ship,
    helm: &mut Helm,
    wells: &[Well],
    (to, lock): (DVec3, f64),
) -> Phase {
    let blocked = blocker(ship, wells).is_some();
    if ship.warp != Warp::Off {
        helm.controls = Controls::default();
        ship.spin = DVec3::ZERO;
        helm.warp |= align::missed(ship, helm.way, (to, lock));
        return next(ship, blocked, false).0;
    }
    let distance = to.length();
    helm.way = waypoint(wells, ship.pos, to / distance, distance - lock);
    let course = helm
        .way
        .map_or(to / distance, |at| (at - ship.pos).normalize());
    let ready = align::settled(ship, course);
    if ready {
        align::snap(ship, course);
    }
    let (phase, switch) = next(ship, blocked, ready);
    if phase != Phase::Waiting {
        helm.controls = Controls {
            turn: aim(ship.rot, course),
            ..Controls::default()
        };
        helm.warp |= switch;
        (ship.throttle, ship.assist) = (0.0, true);
    }
    phase
}

/// Stops the drive that ran on the course before when `settle` asks for it, lets go of every
/// control so that the ship holds its heading, and returns what the autopilot is doing.
fn drop_first(helm: &mut Helm, settle: Settle) -> Phase {
    helm.controls = Controls::default();
    helm.warp |= settle == Settle::Stop;
    Phase::Dropping
}

/// Returns whether the route is a long jump, and the course to the destination as it will be
/// when the ship gets there, so a destination that moves is led and the heading needs no
/// correction on the way. The ship goes on with its own velocity until the drive carries it, so
/// the course is the one from where it will be then. `now` is where the ship is and when, `to`
/// the course now, and `ship` the drive and the velocity.
fn aim_at(
    goal: &Goal,
    universe: &Universe,
    now: (Option<UniPos>, f64),
    (to, ship): (DVec3, (Warp, DVec3)),
) -> (bool, DVec3) {
    let jump = to.length() > JUMP_DISTANCE;
    let cap = if jump { JUMP_CAP } else { CRUISE_CAP };
    let drift = ship.1 * align::spool(ship.0);
    let at = |here: UniPos, ahead: f64| {
        let reached = goal.reached(universe, here + drift, now.1 + ahead);
        reached.map(|(to, _)| to)
    };
    let led = now
        .0
        .and_then(|here| align::led(ship.0, (goal.lock, cap), |ahead| at(here, ahead)));
    (jump, led.unwrap_or(to))
}

/// Flies the route: stops a drive still running on another course and waits for the drop-out,
/// then steers onto the course, asks for the warp drive and ends the route on arrival.
pub fn fly(
    cosmos: Cosmos,
    mut autopilot: ResMut<Autopilot>,
    mut skies: ResMut<Skies>,
    mut helm: ResMut<Helm>,
    mut ship: Single<&mut Ship>,
) {
    let (universe, time) = (&cosmos.universe, cosmos.clock.seconds);
    skies.beacon = None;
    let Some(target) = autopilot.target else {
        return;
    };
    if autopilot.phase == Phase::Climbing || climbs(&ship, Some(target)) {
        return climb(&mut autopilot, &mut ship, &mut helm, &skies, target);
    }
    let goal = autopilot
        .goal
        .get_or_insert_with(|| goal::resolve(universe, target));
    let lock = goal.lock;
    let here = ship_position(universe, ship.frame, ship.pos, time);
    let Some((to, arrived)) = here.and_then(|here| goal.reached(universe, here, time)) else {
        return autopilot.cancel();
    };
    if arrived {
        helm.warp = ship.warp != Warp::Off;
        *autopilot = Autopilot::ARRIVED;
        return;
    }
    let (jump, to) = aim_at(goal, universe, (here, time), (to, (ship.warp, ship.vel)));
    autopilot.jump = jump;
    skies.beacon = autopilot.jump.then(|| beacon(ship.pos, to, lock));
    let settle = autopilot.settle(ship.warp != Warp::Off, time);
    autopilot.phase = match settle {
        Settle::Clear => hold(&mut ship, &mut helm, &skies.wells, (to, lock)),
        _ => drop_first(&mut helm, settle),
    };
}

/// Starts a route when the board computer hands over a destination.
pub fn on_engage(event: On<Engage>, mut autopilot: ResMut<Autopilot>) {
    autopilot.engage(event.0);
}

/// What the route state is read from besides the autopilot.
type Watch<'w, 's> = (
    Res<'w, Skies>,
    Res<'w, Destination>,
    Query<'w, 's, &'static Ship>,
);

/// Shows the map and the HUD how far along the route the ship is, and what keeps the drive
/// from a course that is set. A route that arrived reads as arrived until the course changes.
pub fn report(
    (cosmos, autopilot): (Cosmos, Res<Autopilot>),
    (skies, destination, ships): Watch,
    mut status: ResMut<RouteStatus>,
    mut arrived: Local<Option<Target>>,
) {
    let Some(ship) = ships.iter().next() else {
        return;
    };
    if autopilot.phase != Phase::Arrived {
        *arrived = autopilot.target;
    }
    let other = destination.0.is_some_and(|wanted| Some(wanted) != *arrived);
    let left = autopilot.phase == Phase::Arrived && other;
    let phase = [autopilot.phase, Phase::Idle][usize::from(left)];
    let (universe, time) = (&cosmos.universe, cosmos.clock.seconds);
    let here = ship_position(universe, ship.frame, ship.pos, time);
    let to = autopilot.goal.as_ref().zip(here);
    let to = to.and_then(|(goal, here)| goal.reached(universe, here, time));
    let reading = block::Reading {
        phase,
        block: blocker(ship, &skies.wells).filter(|_| !climbs(ship, destination.0)),
        aligned: to.map_or(0, |(to, _)| aligned(ship.forward(), to)),
        course: destination.0.is_some(),
    };
    status.set_if_neq(block::status(reading));
}
