//! The flight of the ship in cruise: in place of the flight steps, the drive carries it along
//! its track for the universe time the frame covers, its nose along the track and its roof up.
//! The ship keeps the velocity the drive gives it, so the instruments and the other players see
//! it move.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::glide::{self, handover};
use super::near::{Over, over};
use super::skim::{Ask, Band, Skim, Stride, attitude, step};
use super::tune::{ALIGN, PACE};
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

/// Moves `ship` over `over` to the end of `stride`, `dt` seconds on: the planet moved on by its
/// own velocity, the ship by the step on top of it, the nose easing onto the track.
fn fly(ship: &mut Ship, over: &Over, stride: &Stride, dt: f64) {
    ship.pos = over.centre + over.vel * dt + stride.skim.pos;
    ship.vel = over.vel + stride.vel;
    let up = stride.skim.pos.normalize_or(DVec3::Y);
    let wanted = attitude(up, stride.skim.heading, 0.0);
    ship.rot = ship
        .rot
        .slerp(wanted, 1.0 - (-dt / ALIGN).exp())
        .normalize();
    ship.spin = DVec3::ZERO;
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
    let skim = Skim {
        pos: ship.pos - over.centre,
        heading: orbital.heading,
        speed: orbital.speed,
        height: orbital.height,
    };
    let asked = ask(&orbital, &helm, &ship, &over);
    let stride = step(&skim, &Band::of(&over.planet), &asked, dt.max(1e-6));
    if stride.floor {
        return orbital.rest(&mut ship, &over, Some(Note::Floor));
    }
    fly(&mut ship, &over, &stride, dt);
    (orbital.heading, orbital.speed) = (stride.skim.heading, stride.skim.speed);
    orbital.height = stride.skim.height;
    orbital.reading = read(&stride.skim, &over, asked.goal);
    if stride.done {
        orbital.descend(&mut ship, &over);
    }
}
