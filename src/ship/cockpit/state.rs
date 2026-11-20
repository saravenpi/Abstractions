//! The state of the cockpit of one ship: what is asked of the controls, what the instruments
//! read, and what is shown of both after the controls and the needles have followed.

use bevy::prelude::*;

use super::controls::Levers;
use super::readings::{BARS, Readings};
use super::strap::Strap;
use super::tune::SETTLE;

/// The cockpit of one ship, a component of the ship entity. Whoever flies the ship writes
/// what is `asked` and what the instruments `read`; the parts of that ship are moved from what
/// is `shown`, which follows them without a step. Every ship on screen has its own.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Cockpit {
    /// Where the pilot holds the controls right now.
    pub asked: Levers,
    /// What the instruments measure right now.
    pub read: Readings,
    /// Where the controls are drawn, on their way to what is asked.
    pub shown: Levers,
    /// What each bar shows, on its way to what it reads. A bar with nothing to read is dark.
    pub gauges: [Option<f32>; BARS],
    /// The loose strap beside the seat.
    pub strap: Strap,
    /// The player entity that sits in the seat, when one does.
    pub pilot: Option<Entity>,
}

impl Cockpit {
    /// Moves what is shown toward what is asked and read, for `dt` seconds. The controls follow
    /// at [`FOLLOW`](super::tune::FOLLOW), the bars at [`SETTLE`], and the strap swings under
    /// the acceleration that is read.
    pub fn step(&mut self, dt: f32) {
        let share = 1.0 - (-SETTLE * dt).exp();
        self.shown = self.shown.eased(self.asked, dt);
        for (shown, read) in self.gauges.iter_mut().zip(self.read.bars) {
            *shown = match (*shown, read) {
                (_, None) => None,
                (None, Some(_)) => Some(0.0),
                (Some(now), Some(to)) => Some(now + (to - now) * share),
            };
        }
        self.strap = self.strap.stepped(self.read.accel, dt);
    }
}
