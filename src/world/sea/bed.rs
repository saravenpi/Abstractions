//! What the sea floor map is sampled from.

use bevy::prelude::*;

use super::depth;
use crate::world::Terrain;

/// A source of sea floor texels.
pub trait Bed {
    /// Returns the bytes of map texel `texel`: its signed depth and its pond flag.
    fn texel(&self, texel: IVec2) -> [u8; 4];
}

impl Bed for Terrain {
    /// Reads the bed straight from the chart of the terrain, one texel every two chart units.
    fn texel(&self, texel: IVec2) -> [u8; 4] {
        depth::sample(self, texel)
    }
}
