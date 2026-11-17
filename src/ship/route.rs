//! Flying a route: steering onto the course, asking for the warp drive and ending on arrival.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::body::Ship;
use super::flight::Controls;
use super::helm::Helm;
use super::pilot::{Autopilot, Goal, Phase};
use super::sim::Cosmos;
use super::sky::Skies;
use super::steer::{aim, detour};
use super::warp::{Warp, clearance};
use super::wells::{PLANET_LOCK, STAR_LOCK, Well};
use crate::computer::{Engage, RouteStatus, Target, ship_position, target_position};
use crate::space::Frame;
use crate::universe::{GLY, Universe};

/// Distance in metres beyond which a route is a long jump: faster, and blind to the stars on the way.
const JUMP_DISTANCE: f64 = 50.0 * GLY;

/// Lock radii from a planet at which the autopilot drops out, a safe distance for an orbit.
const ARRIVAL: f64 = 1.3;

/// Pull of a star in metres per second squared at the distance where the autopilot drops out.
const SAFE_PULL: f64 = 0.5;

/// Cosine of the largest angle between the nose and the course at which the drive may spool.
const ALIGNED: f64 = 0.9994;

/// Returns the goal for a target: its lock sphere and where to drop out. A galaxy becomes the
/// star nearest to its centre, or its centre when no star is near.
fn resolve(universe: &Universe, target: Target) -> Goal {
    match target {
        Target::Body(id) => {
            let lock = universe.planet(id).radius * PLANET_LOCK;
            Goal {
                target,
                lock,
                arrival: lock * ARRIVAL,
            }
        }
        Target::Star(id) => {
            let star = universe.star(id);
            let lock = star.as_ref().map_or(GLY, |star| star.radius * STAR_LOCK);
            let calm = star.map_or(0.0, |star| (star.mu() / SAFE_PULL).sqrt());
            Goal {
                target,
                lock,
                arrival: calm.max(lock * ARRIVAL),
            }
        }
        Target::Galaxy(id) => match universe.stars_near(id, DVec3::ZERO, 30.0).first() {
            Some(star) => resolve(universe, Target::Star(star.id)),
            None => Goal {
                target,
                lock: GLY,
                arrival: GLY * ARRIVAL,
            },
        },
    }
}

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

/// Flies the route: steers onto the course, asks for the warp drive and ends the route on arrival.
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
    let goal = *autopilot
        .goal
        .get_or_insert_with(|| resolve(universe, target));
    let here = ship_position(universe, ship.frame, ship.pos, time);
    let Some((here, there)) = here.zip(target_position(universe, goal.target, time)) else {
        return autopilot.cancel();
    };
    let (to, warping) = (there - here, ship.warp != Warp::Off);
    let distance = to.length();
    if distance <= goal.arrival {
        helm.warp = warping;
        *autopilot = Autopilot {
            phase: Phase::Arrived,
            ..Autopilot::default()
        };
        return;
    }
    autopilot.jump = distance > JUMP_DISTANCE;
    let beacon = Well {
        pos: ship.pos + to,
        vel: DVec3::ZERO,
        mu: 0.0,
        radius: goal.lock,
        lock: goal.lock,
    };
    skies.beacon = autopilot.jump.then_some(beacon);
    let surface = matches!(ship.frame, Frame::Surface(_));
    let blocked = ship.landed || surface || clearance(&skies.wells, ship.pos) <= 0.0;
    let course = detour(&skies.wells, ship.pos, to / distance, distance - goal.lock);
    let (phase, switch) = next(&ship, blocked, ship.forward().dot(course) > ALIGNED);
    autopilot.phase = phase;
    if phase != Phase::Waiting {
        helm.controls = Controls {
            turn: aim(ship.rot, course),
            ..Controls::default()
        };
        helm.warp |= switch;
        (ship.throttle, ship.assist) = (0.0, true);
    }
}

/// Starts a route when the board computer hands over a destination.
pub fn on_engage(event: On<Engage>, mut autopilot: ResMut<Autopilot>) {
    autopilot.engage(event.0);
}

/// Shows the board computer how far along the route the ship is.
pub fn report(autopilot: Res<Autopilot>, mut status: ResMut<RouteStatus>) {
    let shown = match autopilot.phase {
        Phase::Idle => RouteStatus::Idle,
        Phase::Waiting | Phase::Aligning => RouteStatus::Aligning,
        Phase::Spooling => RouteStatus::Spooling,
        Phase::Cruising => RouteStatus::InWarp,
        Phase::Arrived => RouteStatus::Arrived,
    };
    status.set_if_neq(shown);
}
