//! The autopilot: the route the board computer hands to the ship and how far along it is.

use bevy::math::DVec3;
use bevy::prelude::*;

mod progress;

use super::moons::Standoff;
use super::warp::Tier;
use crate::computer::{Target, target_position};
use crate::render::Sequence;
use crate::space::UniPos;
use crate::universe::{BodyId, StarId, Universe};
pub(super) use progress::Progress;
#[cfg(test)]
pub(super) use progress::SPOOLS;

/// What the autopilot is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// No route is flown.
    #[default]
    Idle,
    /// The drive is locked: the ship is landed, in an atmosphere or too close to a body. The
    /// pilot has to fly it out.
    Waiting,
    /// The drive, still running on the course before, drops out, and the ship holds its heading
    /// until the drop-out has played out. Only then does it turn.
    Dropping,
    /// The ship climbs from the air of the planet it is at to an orbit around it.
    Climbing,
    /// The ship turns its nose onto the course.
    Aligning,
    /// The drive spools.
    Spooling,
    /// The ship is in warp.
    Cruising,
    /// The ship dropped out at the destination.
    Arrived,
}

/// The destination the autopilot flies to, with the distances that depend on it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Goal {
    /// The body flown to. A galaxy is replaced by a star near its centre.
    pub(super) target: Target,
    /// Radius of the lock sphere of the destination in metres.
    pub(super) lock: f64,
    /// Distance in metres at which the ship drops out: from the centre of a planet or a star,
    /// and from the point beside a moon, where [`Standoff::reached`] also counts the space
    /// around the moon.
    pub(super) arrival: f64,
    /// For a moon, the point beside it the route ends at.
    pub(super) beyond: Option<Standoff>,
}

impl Goal {
    /// Returns the metres from a ship at `here` to where the route ends at `time` seconds, the
    /// centre of the destination or the point beside a moon, and whether the ship has arrived.
    pub(super) fn reached(
        &self,
        universe: &Universe,
        here: UniPos,
        time: f64,
    ) -> Option<(DVec3, bool)> {
        let Some(beyond) = &self.beyond else {
            let to = target_position(universe, self.target, time)? - here;
            return Some((to, to.length() <= self.arrival));
        };
        let pos = here - universe.star_position(beyond.moon.id.star)?;
        Some((beyond.point(time) - pos, beyond.reached(pos, time)))
    }
}

/// Seconds a ship holds its heading after its drive was stopped for a new course: the time the
/// drop-out takes on screen.
pub(super) const SETTLE: f64 = Sequence::DROP as f64;

/// What a route does about a drive that ran on the course before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Settle {
    /// The drive runs and is stopped now.
    Stop,
    /// The drive was stopped and the drop-out still plays: the ship holds its heading.
    Hold,
    /// Nothing is left of the course before: the ship may turn and spool.
    Clear,
}

/// The autopilot. The board computer sets a destination with [`Autopilot::engage`].
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Autopilot {
    /// Destination asked for, while a route is flown.
    pub(super) target: Option<Target>,
    /// Destination resolved from the universe.
    pub(super) goal: Option<Goal>,
    /// What the autopilot is doing.
    pub(super) phase: Phase,
    /// The speed tier of the route, set while the drive is off and kept until it stops.
    pub(super) tier: Tier,
    /// Whether the route has yet to look at the drive: one that still runs is stopped first.
    pub(super) fresh: bool,
    /// Universe second until which the ship holds its heading after its drive was stopped for
    /// this route.
    pub(super) settled: f64,
    /// The planet the climb leaves, kept after the ship is out of its air.
    pub(super) leaving: Option<BodyId>,
    /// Whether the pilot stopped the route: the drive that runs is dropped out on the next frame.
    pub(super) halt: bool,
    /// How the route comes on: the nearest it has been and the spools since it came nearer.
    pub(super) progress: Progress,
    /// Whether the route was dropped for spooling again and again without coming nearer.
    pub(super) lost: bool,
    /// Seconds the route has read as lost.
    pub(super) lost_for: f32,
}

impl Autopilot {
    /// The autopilot at rest after a route that arrived.
    pub(super) const ARRIVED: Autopilot = Autopilot {
        target: None,
        goal: None,
        phase: Phase::Arrived,
        tier: Tier::Cruise,
        fresh: false,
        settled: 0.0,
        leaving: None,
        halt: false,
        progress: Progress {
            best: None,
            spools: 0,
        },
        lost: false,
        lost_for: 0.0,
    };

    /// Starts a route to `target`: a planet, a moon, a star or a galaxy. The route already
    /// flown to that target goes on as it is.
    pub fn engage(&mut self, target: Target) {
        if self.target == Some(target) {
            return;
        }
        *self = Self {
            target: Some(target),
            phase: Phase::Waiting,
            fresh: true,
            tier: self.tier,
            ..Self::default()
        };
    }

    /// Returns what a route does about the drive of the course before, at universe second
    /// `time`, when the drive is `running`, spooling or engaged.
    ///
    /// A new route that finds the drive running stops it, then holds the heading for the
    /// [`SETTLE`] seconds the drop-out takes on screen. The ship never turns in warp.
    pub(super) fn settle(&mut self, running: bool, time: f64) -> Settle {
        self.settled = self.settled.min(time + SETTLE);
        if std::mem::take(&mut self.fresh) && running {
            self.settled = time + SETTLE;
            return Settle::Stop;
        }
        if time < self.settled {
            return Settle::Hold;
        }
        Settle::Clear
    }

    /// Drops the route and hands the ship back to the pilot.
    pub fn cancel(&mut self) {
        if self.target.is_some() {
            *self = Self::default();
        }
    }

    /// Returns the destination of the route being flown.
    pub fn target(&self) -> Option<Target> {
        self.target
    }

    /// Returns what the autopilot is doing.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Returns the star of the destination, once the route is resolved to a body or a star.
    pub fn star(&self) -> Option<StarId> {
        match self.goal.as_ref()?.target {
            Target::Body(id) => Some(id.star),
            Target::Star(id) => Some(id),
            Target::Galaxy(_) => None,
        }
    }

    /// Returns true while the route is a long jump, in a faster tier and blind to the stars.
    pub fn jump(&self) -> bool {
        self.tier.blind()
    }
}
