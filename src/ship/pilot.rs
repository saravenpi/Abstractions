//! The autopilot: the route the board computer hands to the ship and how far along it is.

use bevy::prelude::*;

use super::moons::Standoff;
use crate::computer::{Target, target_position};
use crate::space::UniPos;
use crate::universe::{StarId, Universe};

/// What the autopilot is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// No route is flown.
    #[default]
    Idle,
    /// The drive is locked: the ship is landed, in an atmosphere or too close to a body. The
    /// pilot has to fly it out.
    Waiting,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Goal {
    /// The body flown to. A galaxy is replaced by a star near its centre.
    pub(super) target: Target,
    /// Radius of the lock sphere of the destination in metres.
    pub(super) lock: f64,
    /// Distance in metres at which the ship drops out: from the centre of a planet or a star,
    /// from the point beside a moon.
    pub(super) arrival: f64,
    /// For a moon, the point beside it the route ends at.
    pub(super) beyond: Option<Standoff>,
}

impl Goal {
    /// Returns the universe position the route ends at, at `time` seconds: the centre of the
    /// destination, or the point beside a moon.
    pub(super) fn point(&self, universe: &Universe, time: f64) -> Option<UniPos> {
        let centre = target_position(universe, self.target, time)?;
        Some(
            self.beyond
                .map_or(centre, |beyond| centre + beyond.offset(time)),
        )
    }
}

/// The autopilot. The board computer sets a destination with [`Autopilot::engage`].
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Autopilot {
    /// Destination asked for, while a route is flown.
    pub(super) target: Option<Target>,
    /// Destination resolved from the universe.
    pub(super) goal: Option<Goal>,
    /// What the autopilot is doing.
    pub(super) phase: Phase,
    /// Whether the route is a long jump, which uses the faster warp tier.
    pub(super) jump: bool,
}

impl Autopilot {
    /// Starts a route to `target`: a planet, a moon, a star or a galaxy.
    pub fn engage(&mut self, target: Target) {
        *self = Self {
            target: Some(target),
            phase: Phase::Waiting,
            ..Self::default()
        };
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
        match self.goal?.target {
            Target::Body(id) => Some(id.star),
            Target::Star(id) => Some(id),
            Target::Galaxy(_) => None,
        }
    }

    /// Returns true while the route is a long jump.
    pub fn jump(&self) -> bool {
        self.jump
    }
}
