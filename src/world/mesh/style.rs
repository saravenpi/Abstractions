//! How a patch of terrain is drawn: its floor, its skirts, whether its triangles are toned apart and the
//! ground cover whose colour its grass takes.

use super::tune::TONE_REACH;
use crate::world::SEA_LEVEL;
use crate::world::lands::{Lands, Verdure};

/// How far below sea level a coarse mesh draws its flat sea, so that the real sea covers it.
pub(super) const SEA_DROP: f32 = 1.5;

/// How a patch is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Lowest height drawn: lower ground is covered by a flat sea at this height.
    pub floor: f32,
    /// How far the skirts hang below the edges, in metres. Zero means no skirts.
    pub skirt: f32,
    /// The ground rules of the planet, which give the surfaces and their colours.
    pub lands: Lands,
    /// Whether the triangles are toned apart: only cells no wider than [`TONE_REACH`] are.
    pub toned: bool,
    /// The ground cover whose mean colour grass takes, where a turf grows.
    pub verdure: Option<Verdure>,
}

impl Style {
    /// Returns the style of a detail chunk on `lands`: the real sea bed and the skirts of the 2 m grid.
    pub fn detail(lands: Lands) -> Style {
        Style {
            floor: f32::NEG_INFINITY,
            skirt: 3.0,
            lands,
            toned: true,
            verdure: None,
        }
    }

    /// Returns the style of a coarse mesh on `lands` with cells `cell` metres wide: a flat sea where the
    /// planet has one, and skirts deep enough to hide the cracks along a finer neighbour.
    pub fn coarse(cell: f32, lands: Lands) -> Style {
        let floor = match lands.has_sea() {
            true => SEA_LEVEL - SEA_DROP,
            false => f32::NEG_INFINITY,
        };
        Style {
            floor,
            skirt: 3.0 + cell * 0.5,
            lands,
            toned: cell <= TONE_REACH,
            verdure: None,
        }
    }

    /// Returns this style for cells `cell` metres wide with the ground cover `verdure`.
    pub fn grown(self, cell: f32, verdure: Option<Verdure>) -> Style {
        Style {
            toned: self.toned && cell <= TONE_REACH,
            verdure,
            ..self
        }
    }
}

/// Returns a stable value in `[0.92, 1.08]` for a grid cell and triangle index.
pub fn tone(i: usize, j: usize, half: usize) -> f32 {
    let h = (i as u32).wrapping_mul(73_856_093)
        ^ (j as u32).wrapping_mul(19_349_663)
        ^ (half as u32 * 83_492_791);
    let h = (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D);
    0.92 + ((h >> 10) & 0xFF) as f32 / 255.0 * 0.16
}
