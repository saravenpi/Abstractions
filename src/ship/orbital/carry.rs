//! The flight of the ship in cruise: in place of the flight steps, the drive carries it along
//! its track for the universe time the frame covers. The ship first turns onto the course at a
//! bounded rate, easing in and out, and the speed builds once its nose is on it.
//! The ship keeps the velocity the drive gives it, so the instruments and the other players see
//! it move.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::glide::{self, handover};
use super::near::{Over, over};
use super::skim::{Ask, Band, Skim, Stride, attitude, course, step};
use super::tune::{ALIGNED, PACE};
use super::{Note, Orbital, Reading};
use crate::ship::body::Ship;
use crate::ship::flight::Report;
use crate::ship::helm::Helm;
use crate::ship::sim::Pace;
use crate::ship::sky::Skies;
use crate::ship::warp::RAMP;
use crate::space::Clock;

/// Returns what the helm and the target ask of the cruise of `ship` over `over`: toward a target
/// the lever is full, and the vertical stick and the pitch move the height held.
fn ask(orbital: &Orbital, helm: &Helm, ship: &Ship, over: &Over) -> Ask {
    let goal = orbital.goal.map(|goal| over.toward(goal));
    Ask {
        throttle: [ship.throttle, 1.0][usize::from(goal.is_some())],
        climb: (helm.controls.lift + helm.controls.turn.x).clamp(-1.0, 1.0),
        yaw: helm.controls.turn.y,
        goal,
    }
}

/// Returns what the HUD reads of a cruise at `skim` over `over` toward the unit direction
/// `goal`: the arc to it, the arc to the start of the glide, and the time at full lever then
/// along the glide.
fn read(skim: &Skim, over: &Over, goal: Option<DVec3>) -> Option<Reading> {
    let goal = goal?;
    let band = Band::of(&over.planet);
    let arc = skim.pos.angle_between(goal) * band.radius;
    let sloped = handover(skim.height).min(arc);
    let descent = arc - sloped;
    let eta = descent / band.top(skim.height) + RAMP + sloped / (PACE * glide::top(&over.planet));
    Some(Reading {
        arc,
        eta,
        descent: Some(descent),
    })
}

/// Moves `ship` over `over` to the end of `stride`, `dt` seconds on, turned to `rot`: the planet
/// moved on by its own velocity, the ship by the step on top of it. Returns the turn of the
/// vertical over the step, which carries the nose and the course round the planet with the ship.
fn fly(ship: &mut Ship, over: &Over, stride: &Stride, (rot, dt): (DQuat, f64)) -> DQuat {
    let from = (ship.pos - over.centre).normalize_or(DVec3::Y);
    let carried = DQuat::from_rotation_arc(from, stride.skim.pos.normalize_or(from));
    ship.pos = over.centre + over.vel * dt + stride.skim.pos;
    ship.vel = over.vel + stride.vel;
    ship.rot = (carried * rot).normalize();
    ship.spin = DVec3::ZERO;
    carried
}

/// Flies one frame of `dt` seconds of the cruise of `ship` over `over`, as `asked`: the ship
/// turns onto the course at its own rate, the speed builds only once the nose is on it, and the
/// drive carries the ship along the way the nose points. Returns the step, or nothing when the
/// pilot asks to go lower than the floor and the cruise ends there.
pub(super) fn advance(
    orbital: &mut Orbital,
    ship: &mut Ship,
    over: &Over,
    asked: &Ask,
    dt: f64,
) -> Option<Stride> {
    let rel = ship.pos - over.centre;
    let up = rel.normalize_or(DVec3::Y);
    let skim = Skim {
        pos: rel,
        heading: orbital.heading,
        speed: orbital.speed,
        height: orbital.height,
    };
    let way = course(&skim, asked, dt);
    let rot = orbital.turn(ship.rot, attitude(up, way, 0.0), dt);
    let nose = rot * DVec3::NEG_Z;
    let flat = (nose - up * nose.dot(up)).normalize_or(way);
    orbital.aligned |= flat.angle_between(way) < ALIGNED;
    let held = Ask {
        throttle: asked.throttle * f64::from(u8::from(orbital.aligned)),
        ..*asked
    };
    let skim = Skim {
        heading: flat,
        ..skim
    };
    let stride = step(&skim, &Band::of(&over.planet), &held, dt.max(1e-6));
    if stride.floor {
        return None;
    }
    let carried = fly(ship, over, &stride, (rot, dt));
    (orbital.heading, orbital.speed) = (carried * way, stride.skim.speed);
    orbital.height = stride.skim.height;
    Some(stride)
}

/// Carries the ship in cruise through the universe time of the frame, as the flight steps would
/// have stepped it. At the start of the glide it hands over to the glide; at the floor, a pilot
/// who asks to go lower is left at rest over the ground.
pub fn carry(
    (clock, skies, helm): (Res<Clock>, Res<Skies>, Res<Helm>),
    mut pace: ResMut<Pace>,
    mut orbital: ResMut<Orbital>,
    mut ship: Single<&mut Ship>,
) {
    let dt = pace.lag;
    (pace.carry, pace.report) = (0.0, Report::default());
    let Some(over) = over(&ship, &skies, (clock.seconds - dt, 0.0)) else {
        return orbital.off(None);
    };
    let asked = ask(&orbital, &helm, &ship, &over);
    let Some(stride) = advance(&mut orbital, &mut ship, &over, &asked, dt) else {
        return orbital.rest(&mut ship, &over, Some(Note::Floor));
    };
    orbital.reading = read(&stride.skim, &over, asked.goal);
    if stride.done {
        orbital.descend(&mut ship, &over);
    }
}
