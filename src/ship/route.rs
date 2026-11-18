//! Flying a route: steering onto the course, asking for the warp drive and ending on arrival.

mod goal;

pub(super) use goal::ARRIVAL;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::flight::Controls;
use super::helm::Helm;
use super::pilot::{Autopilot, Phase, Settle};
use super::sim::Cosmos;
use super::sky::Skies;
use super::steer::{aim, detour};
use super::warp::{Warp, clearance};
use super::wells::Well;
use crate::computer::{Engage, RouteStatus, ship_position};
use crate::space::Frame;
use crate::universe::GLY;

/// Distance in metres beyond which a route is a long jump: faster, and blind to the stars on the way.
const JUMP_DISTANCE: f64 = 50.0 * GLY;

/// Cosine of the largest angle between the nose and the course at which the drive may spool.
const ALIGNED: f64 = 0.9994;

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
    }
}

/// Turns the ship onto the course to a destination `to` metres away with a lock sphere of
/// `lock` metres, asks for the drive once aligned and returns what the autopilot is doing.
///
/// While the drive is locked the pilot keeps the controls.
pub(super) fn hold(
    ship: &mut Ship,
    helm: &mut Helm,
    wells: &[Well],
    (to, lock): (DVec3, f64),
) -> Phase {
    let distance = to.length();
    let surface = matches!(ship.frame, Frame::Surface(_));
    let blocked = ship.landed || surface || clearance(wells, ship.pos) <= 0.0;
    let course = detour(wells, ship.pos, to / distance, distance - lock);
    let (phase, switch) = next(ship, blocked, ship.forward().dot(course) > ALIGNED);
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
        *autopilot = Autopilot {
            phase: Phase::Arrived,
            ..Autopilot::default()
        };
        return;
    }
    autopilot.jump = to.length() > JUMP_DISTANCE;
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

/// Shows the board computer how far along the route the ship is.
pub fn report(autopilot: Res<Autopilot>, mut status: ResMut<RouteStatus>) {
    let shown = match autopilot.phase {
        Phase::Idle => RouteStatus::Idle,
        Phase::Waiting | Phase::Dropping | Phase::Aligning => RouteStatus::Aligning,
        Phase::Spooling => RouteStatus::Spooling,
        Phase::Cruising => RouteStatus::InWarp,
        Phase::Arrived => RouteStatus::Arrived,
    };
    status.set_if_neq(shown);
}
