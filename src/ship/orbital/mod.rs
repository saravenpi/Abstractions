//! Orbital cruise: above the air of a planet, inside its lock sphere where the warp drive cannot
//! run, the drive carries the ship around the planet at a height it holds, half a turn in about
//! a minute and a half. Toward a target, such as the home base, it stops where a glide of
//! [`tune::GLIDE_ANGLE`] reaches the aim, and the autopilot glides the ship down into the air at
//! a speed that keeps the nose dark, to a few kilometres above and short of the target. The
//! landing is the pilot's.
//!
//! The warp key starts and stops a cruise; Fly home on the map flies home through it. The cruise never
//! goes into the air: at its floor, a pilot who asks to go lower is dropped at rest over the
//! ground first, at a speed no entry can heat.

mod carry;
mod glide;
mod home;
mod near;
mod shift;
mod skim;
mod tune;
mod watch;

#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests_air;
#[cfg(test)]
mod tests_cross;
#[cfg(test)]
mod tests_glide;
#[cfg(test)]
mod tests_home;
#[cfg(test)]
mod tests_turn;

pub(super) use carry::carry;
pub(super) use home::on_homeward;
pub(super) use watch::watch;

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::universe::BodyId;

/// What the orbital cruise is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Regime {
    /// No cruise and no glide.
    #[default]
    Off,
    /// The drive carries the ship around the planet.
    Cruise,
    /// The autopilot glides the ship down into the air toward the target.
    Glide,
}

/// Why the last cruise or glide ended, for the prompt line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    /// The pilot asked to go into the air: the drive dropped the ship at rest over the ground.
    Floor,
    /// The glide reached its end: the target is ahead, for the pilot to land at.
    Arrived,
    /// Home was asked for from the air near it: it is close enough to fly to.
    Near,
}

/// A target asked for before the ship could cruise to it: on the way, or still in the air.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Pending {
    /// The planet the target is on.
    pub body: BodyId,
    /// The target, in metres from the centre of the planet in its own turning frame.
    pub goal: DVec3,
    /// Whether a route or a climb toward it is under way.
    pub flown: bool,
}

/// What the HUD reads of a cruise or a glide toward a target.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Reading {
    /// Metres along the ground to the target.
    pub arc: f64,
    /// Rough seconds to the target.
    pub eta: f64,
    /// Metres along the ground to the point where the glide begins, during the cruise.
    pub descent: Option<f64>,
}

/// The orbital cruise of the ship: the regime, its target and what the HUD shows of it.
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct Orbital {
    /// What it is doing.
    pub(super) regime: Regime,
    /// The target on the planet below, in metres from its centre in its turning frame.
    pub(super) goal: Option<DVec3>,
    /// A target asked for before the ship could cruise.
    pub(super) pending: Option<Pending>,
    /// Height held, in metres above sea level.
    pub(super) height: f64,
    /// Speed of the cruise in metres per second.
    pub(super) speed: f64,
    /// Unit direction of the course of the cruise, in the axes of the frame of the ship.
    pub(super) heading: DVec3,
    /// Speed in radians per second at which the ship turns onto the course.
    pub(super) slew: f64,
    /// Whether the nose has come onto the course, so that the cruise builds its speed.
    pub(super) aligned: bool,
    /// Whether the ship could start a cruise now.
    pub(super) ready: bool,
    /// The distances and the time to the target.
    pub(super) reading: Option<Reading>,
    /// Why the last cruise ended, and the seconds the prompt still says so.
    pub(super) note: Option<(Note, f64)>,
}

impl Orbital {
    /// Returns what the cruise is doing.
    pub fn regime(&self) -> Regime {
        self.regime
    }

    /// Returns whether the ship could start a cruise now: above the air and inside the lock
    /// sphere of a planet, with no cruise running.
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// Returns the distances and the time to the target of the cruise or the glide.
    pub fn reading(&self) -> Option<Reading> {
        self.reading
    }

    /// Returns why the last cruise ended, while the prompt still says so.
    pub fn note(&self) -> Option<Note> {
        self.note.map(|(note, _)| note)
    }

    /// Returns whether the ship climbs or warps toward a target to cruise to.
    pub fn pending(&self) -> bool {
        self.pending.is_some_and(|pending| pending.flown)
    }
}

/// Run condition: the drive carries the ship in cruise.
pub fn cruising(orbital: Res<Orbital>) -> bool {
    orbital.regime == Regime::Cruise
}
