//! The keys and the autopilot of the orbital cruise, before the flight steps: the warp key starts
//! a cruise where the warp drive is locked and stops one, a target asked for starts a cruise once
//! the ship is free, and the glide flies the ship down into the air until the pilot pitches up,
//! presses the warp key or the glide ends.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::glide::{Fix, order};
use super::home::{Move, answer, apply, spot};
use super::near::{Over, free, over};
use super::tune::ABORT;
use super::{Note, Orbital, Reading, Regime};
use crate::computer::Destination;
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::ship::helm::Helm;
use crate::ship::pilot::Autopilot;
use crate::ship::sim::Pace;
use crate::ship::sky::Skies;
use crate::ship::steer::aim;
use crate::space::{Clock, Frame};

/// What the watch reads and writes besides the cruise and the ship.
#[derive(SystemParam)]
pub struct Deck<'w> {
    /// Frame time.
    time: Res<'w, Time>,
    /// Universe time.
    clock: Res<'w, Clock>,
    /// What surrounds the ship.
    skies: Res<'w, Skies>,
    /// Bookkeeping of the flight steps, with the lag of the bodies.
    pace: Res<'w, Pace>,
    /// What the pilot asks for.
    helm: ResMut<'w, Helm>,
    /// The route of the board computer.
    autopilot: ResMut<'w, Autopilot>,
    /// The course the board computer is set to.
    course: ResMut<'w, Destination>,
}

/// With no cruise: the warp key of the pilot starts one where the ship is free, and a target
/// asked for starts one, or the climb to it, when it can. The warp key of the pilot forgets a
/// target asked for, and away from a planet it still works the warp drive, as does a route.
fn idle(deck: &mut Deck, orbital: &mut Orbital, ship: &mut Ship, over: Option<&Over>, free: bool) {
    let pilot = deck.helm.warp && deck.autopilot.target().is_none();
    if pilot {
        orbital.pending = None;
    }
    if let Some(over) = over.filter(|_| pilot && free) {
        deck.helm.warp = false;
        return orbital.start(ship, over, None);
    }
    let Some(pending) = orbital.pending else {
        return;
    };
    let spot = spot(ship, over, &pending, free);
    let motion = answer(&pending, spot, deck.autopilot.target());
    deck.helm.warp &= motion != Move::Cruise;
    let wheel = (orbital, &mut *deck.autopilot, &mut *deck.course);
    apply(motion, pending, wheel, ship, over);
}

/// In cruise: the warp key stops it at rest over the ground, and so does a route the board
/// computer starts.
fn cruise(deck: &mut Deck, orbital: &mut Orbital, ship: &mut Ship, over: Option<&Over>) {
    let Some(over) = over else {
        return orbital.off(None);
    };
    if std::mem::take(&mut deck.helm.warp) {
        return orbital.rest(ship, over, None);
    }
    if deck.autopilot.target().is_some() {
        orbital.rest(ship, over, None);
    }
}

/// Returns the reading of a glide of `ship` at `rel` metres from the centre of `over` toward the
/// unit direction `goal`: the arc to it and the time at the speed over the ground.
fn read(ship: &Ship, over: &Over, rel: DVec3, goal: DVec3) -> Reading {
    let arc = rel.angle_between(goal) * over.planet.radius;
    let up = rel.normalize_or(DVec3::Y);
    let ground = ship.vel - over.vel - over.ground(rel);
    let flat = (ground - up * ground.dot(up)).length();
    Reading {
        arc,
        eta: arc / flat.max(1.0),
        descent: None,
    }
}

/// In the glide: flies the ship along the glide path for the frame, or gives it back to the
/// pilot when the pilot pitches up or presses the warp key, a route starts, or the glide ends.
fn glide(deck: &mut Deck, orbital: &mut Orbital, ship: &mut Ship, over: Option<&Over>) {
    let (Some(over), Some(goal)) = (over, orbital.goal) else {
        return orbital.off(None);
    };
    let taken = std::mem::take(&mut deck.helm.warp) || deck.helm.controls.turn.x > ABORT;
    if taken || deck.autopilot.target().is_some() {
        return orbital.off(None);
    }
    let rel = ship.pos - over.centre;
    let ground = match ship.frame {
        Frame::Surface(_) => deck.skies.height(ship.pos),
        _ => 0.0,
    };
    let toward = over.toward(goal);
    let fix = Fix {
        pos: rel,
        frame: over.ground(rel),
        goal: toward,
        ground,
    };
    let order = order(&fix, &over.planet);
    orbital.reading = Some(read(ship, over, rel, toward));
    if order.done {
        deck.helm.controls = Controls::default();
        return orbital.off(Some(Note::Arrived));
    }
    deck.helm.controls = Controls {
        turn: aim(ship.rot, order.aim),
        ..Controls::default()
    };
    (ship.throttle, ship.assist) = (order.throttle, true);
}

/// Watches the keys and flies the autopilot of the cruise for the frame, after the helm and the
/// route of the board computer.
pub fn watch(mut deck: Deck, mut orbital: ResMut<Orbital>, mut ship: Single<&mut Ship>) {
    orbital.tick(deck.time.delta_secs_f64());
    let time = (deck.clock.seconds, deck.pace.lag);
    let over = over(&ship, &deck.skies, time);
    let free = over.as_ref().is_some_and(|over| free(&ship, over));
    orbital.ready = free && orbital.regime == Regime::Off;
    match orbital.regime {
        Regime::Off => idle(&mut deck, &mut orbital, &mut ship, over.as_ref(), free),
        Regime::Cruise => cruise(&mut deck, &mut orbital, &mut ship, over.as_ref()),
        Regime::Glide => glide(&mut deck, &mut orbital, &mut ship, over.as_ref()),
    }
}
