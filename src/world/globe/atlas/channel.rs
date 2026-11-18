//! Channels: what flows down a planet's high ground, how wide and deep its bed grows, and its cross profile.

use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::world::islands::smooth;
use crate::world::surface::Surface;

/// Grid steps after which a channel has grown to its full width and depth.
const GROWN: f32 = 20.0;

/// What runs down the high ground of a planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flowing {
    /// Nothing: no channel is carved.
    Nothing,
    /// Rivers of water from the wet highlands, on worlds of liquid water seas.
    Water,
    /// Dark flows of molten rock from the high ground to the lava seas.
    Lava,
}

impl Flowing {
    /// Returns what flows on a planet of `record` that has a sea when `sea` is set.
    pub fn of(record: &PlanetRecord, sea: bool) -> Self {
        match (record.class, record.water) {
            _ if !sea => Flowing::Nothing,
            (PlanetClass::Ocean | PlanetClass::Island, Water::Liquid) => Flowing::Water,
            (PlanetClass::Lava, _) => Flowing::Lava,
            _ => Flowing::Nothing,
        }
    }

    /// Returns the half width and the depth in metres of a channel `step` grid steps from its source.
    ///
    /// Rivers grow from 6 to 24 m wide and 2 to 6 m deep, lava flows from 4 to 10 m wide and 1 to 2 m deep.
    pub fn size(self, step: usize) -> (f32, f32) {
        let grown = smooth(0.0, GROWN, step as f32);
        match self {
            Flowing::Lava => ((4.0 + 6.0 * grown) * 0.5, 1.0 + grown),
            _ => ((6.0 + 18.0 * grown) * 0.5, 2.0 + 4.0 * grown),
        }
    }

    /// Returns the surface of a channel bed.
    pub fn bed(self) -> Surface {
        match self {
            Flowing::Lava => Surface::Rock,
            _ => Surface::River,
        }
    }
}

/// Returns how much a channel of half width `half` pulls the ground towards its bed `distance` metres
/// from its middle: 1 across a flat bottom, falling smoothly to 0 at the top of its banks.
pub fn pull(distance: f32, half: f32) -> f32 {
    smooth(1.7 * half, 0.4 * half, distance)
}
