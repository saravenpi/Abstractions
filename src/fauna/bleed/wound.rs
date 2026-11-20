//! Wounds: where a blow opened an animal, and how much blood leaves through the opening.

use bevy::prelude::*;

use super::tune::{CLOSED, MOST, PLUG};
use crate::life::Species;
use crate::life::blood::{Blood, clot, gush, heart_rate, pressure, seep, volume};

/// One opening in the body of an animal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    /// Index of the part it is on.
    pub part: u8,
    /// Where it is, in the frame of the animal.
    pub at: Vec3,
    /// Unit direction the surface faces there, in the frame of the animal.
    pub normal: Vec3,
    /// Width of the cut in metres.
    pub width: f32,
    /// Depth of the cut in metres.
    pub depth: f32,
    /// True when it reached a major vessel.
    pub major: bool,
    /// Seconds since it was cut.
    pub age: f32,
    /// Blood that left but was not thrown as a drop yet, in cubic metres.
    pub owed: f32,
    /// True when another player's spear stands in it: it flows as a plugged cut does.
    pub plugged: bool,
}

/// A cut opened by a blow of this machine: the animal and the cut, for the network to tell.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Bled {
    /// Root of the animal that was cut.
    pub victim: Entity,
    /// The cut.
    pub cut: Cut,
}

/// The wounds of an animal and the blood it lost through them.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Wound {
    /// The open cuts.
    pub cuts: [Option<Cut>; MOST],
    /// Blood lost in all, in cubic metres.
    pub lost: f32,
}

impl Wound {
    /// Opens `cut`: in a free slot, else in place of the oldest.
    pub fn add(&mut self, cut: Cut) {
        let free = self.cuts.iter().position(Option::is_none);
        let oldest = || {
            let age = |c: &Option<Cut>| c.map_or(0.0, |c| c.age);
            let all = self.cuts.iter().enumerate();
            all.max_by(|a, b| age(a.1).total_cmp(&age(b.1)))
                .map(|(i, _)| i)
        };
        if let Some(slot) = free.or_else(oldest) {
            self.cuts[slot] = Some(cut);
        }
    }

    /// Returns true when some cut is open.
    pub fn open(&self) -> bool {
        self.cuts.iter().any(Option::is_some)
    }
}

/// What the blood of one body does: how much it holds and how hard it is pushed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circuit {
    /// Mass of the animal in kilograms.
    pub mass: f32,
    /// Pressure in the arteries, in pascals.
    pub pressure: f32,
    /// Blood it holds when whole, in cubic metres.
    pub volume: f32,
    /// Beats of the heart per second.
    pub beat: f32,
}

impl Circuit {
    /// Returns the circuit of an animal of `species` with `blood` on a planet of `gravity`.
    pub fn of(species: &Species, blood: &Blood, gravity: f32) -> Self {
        let mass = species.history.mass;
        Self {
            mass,
            pressure: pressure(gravity, species.body.height, blood.closed),
            volume: volume(mass),
            beat: heart_rate(mass),
        }
    }

    /// Returns the share of the pressure that is left after `lost` cubic metres are gone: it falls
    /// to nothing at the lethal loss.
    pub fn left(&self, lost: f32) -> f32 {
        let lethal = self.volume * crate::life::blood::tune::LETHAL;
        (1.0 - lost / lethal.max(f32::EPSILON)).clamp(0.0, 1.0)
    }

    /// Returns the mean flow of `cut` in cubic metres per second after `lost` cubic metres are gone:
    /// an artery cut gushes at the pressure that is left, a shallow cut seeps, both clot with time,
    /// and a spear in the wound `plugged` it. A flow under [`CLOSED`] is none.
    pub fn rate(&self, cut: &Cut, lost: f32, plugged: bool) -> f32 {
        let left = self.left(lost);
        let open = if cut.major {
            gush(cut.width, self.mass, self.pressure * left)
        } else {
            seep(cut.width, cut.depth) * left
        };
        let q = open * clot(cut.age, cut.major) * if plugged { PLUG } else { 1.0 };
        if q < CLOSED { 0.0 } else { q }
    }
}
