//! Flying home through the cruise: Fly home on the map, on or above the home planet, asks for the base,
//! and the ship cruises to it at once above the air, climbs out of the air first when the base
//! is far, or warps to the planet first when the computer set that course. Near the base in the
//! air there is nothing to cruise: the marker shows the way.

use bevy::prelude::*;

use super::glide::handover;
use super::near::Over;
use super::skim::Band;
use super::{Note, Orbital, Pending};
use crate::computer::{Destination, Homeward, Target};
use crate::home::Base;
use crate::ship::body::Ship;
use crate::ship::pilot::Autopilot;
use crate::space::Frame;

/// Where the ship is against the planet of a target it is asked to cruise to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Spot {
    /// Above the air of the planet, free to cruise.
    Over,
    /// In the air of the planet, `far` when the target lies past where a glide from the cruise
    /// would begin.
    Air {
        /// Whether a cruise is worth flying to the target.
        far: bool,
    },
    /// Over the planet with the warp drive running or the ship landed: it will be free soon.
    Busy,
    /// Away from the planet.
    Away,
}

/// What to do about a target asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Move {
    /// Start the cruise to it.
    Cruise,
    /// Hand the ship to the autopilot to climb out of the air, then cruise.
    Climb,
    /// The route the computer flies goes to its planet: cruise on arrival.
    Follow,
    /// Set the course to its planet and cruise on arrival.
    Route,
    /// Wait for the ship to be free.
    Wait,
    /// Forget it, with what to say about it.
    Drop(Option<Note>),
}

/// Returns where `ship` is against the planet of `pending`, given the planet it is `over` and
/// whether it is `free` to cruise there.
pub(super) fn spot(ship: &Ship, over: Option<&Over>, pending: &Pending, free: bool) -> Spot {
    let Some(over) = over.filter(|over| over.planet.id == pending.body) else {
        return Spot::Away;
    };
    let arc =
        (ship.pos - over.centre).angle_between(over.toward(pending.goal)) * over.planet.radius;
    match ship.frame {
        Frame::Surface(_) => Spot::Air {
            far: arc > handover(Band::of(&over.planet).floor),
        },
        _ if free => Spot::Over,
        _ => Spot::Busy,
    }
}

/// Returns what to do about `pending` from `spot`, while the autopilot flies to `route`.
pub(super) fn answer(pending: &Pending, spot: Spot, route: Option<Target>) -> Move {
    let ours = route == Some(Target::Body(pending.body));
    match (spot, pending.flown) {
        (Spot::Over, _) => Move::Cruise,
        (Spot::Busy, _) => Move::Wait,
        (Spot::Air { far: true }, false) => Move::Climb,
        (Spot::Air { far: false }, false) => Move::Drop(Some(Note::Near)),
        (Spot::Away, false) if ours => Move::Follow,
        (Spot::Away, false) => Move::Route,
        _ if route.is_none() => Move::Drop(None),
        _ => Move::Wait,
    }
}

/// Does `motion` for `pending`: starts the cruise of `ship` over `over`, hands the climb or the
/// course to the autopilot, or forgets the target.
pub(super) fn apply(
    motion: Move,
    pending: Pending,
    (orbital, autopilot, course): (&mut Orbital, &mut Autopilot, &mut Destination),
    ship: &mut Ship,
    over: Option<&Over>,
) {
    let flown = Some(Pending {
        flown: true,
        ..pending
    });
    match (motion, over) {
        (Move::Cruise, Some(over)) => {
            autopilot.cancel();
            course.0 = course.0.filter(|to| *to != Target::Body(pending.body));
            orbital.start(ship, over, Some(pending.goal));
        }
        (Move::Climb, _) => {
            autopilot.engage(Target::Body(pending.body));
            orbital.pending = flown;
        }
        (Move::Route, _) => {
            course.0 = Some(Target::Body(pending.body));
            autopilot.engage(Target::Body(pending.body));
            orbital.pending = flown;
        }
        (Move::Follow, _) => orbital.pending = flown,
        (Move::Drop(note), _) => {
            orbital.pending = None;
            orbital.off(note);
        }
        _ => {}
    }
}

/// Takes the home base as the target to cruise to when Fly home on the map asks for it on or above its
/// planet. Without a known base there is nothing to cruise to.
pub fn on_homeward(event: On<Homeward>, base: Option<Res<Base>>, mut orbital: ResMut<Orbital>) {
    let Some(base) = base.filter(|base| base.body == event.0) else {
        return;
    };
    orbital.pending = Some(Pending {
        body: base.body,
        goal: base.home(),
        flown: false,
    });
}
