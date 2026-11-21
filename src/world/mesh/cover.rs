//! The turf cover a mesh bakes into the colour of its grass: a chunk's own carpet, or a lattice for ground wider than
//! the carpets that grow on it.

use bevy::prelude::*;

use super::lod::Lod;
use super::span::Span;
use crate::world::Terrain;
use crate::world::scatter::{Carpet, Spread};

/// Points along one side of the lattice on which a distant chunk reads how thick its turf grows.
const FAR_LATTICE: usize = 3;

/// Returns the turf of chunk `coord` as `lod` reads it: the carpet of the chunk, or for a distant chunk, whose
/// ground has no turf of its own, a lattice of its thickness that costs a few reads and not a climate grid.
pub(super) fn of(
    terrain: &Terrain,
    coord: IVec2,
    span: &Span,
    lod: Lod,
) -> (Option<Carpet>, Option<Spread>) {
    if lod != Lod::Distant {
        return (Carpet::of(terrain, coord), None);
    }
    let corners = (span.chart(0, 0, 0.0), span.chart(span.n, span.n, 0.0));
    (None, Spread::over(terrain, corners, FAR_LATTICE))
}

/// Returns how thick the turf grows at chart position `w` of the carpet or the spread, 0 when there is neither.
pub(super) fn at((carpet, spread): (&Option<Carpet>, &Option<Spread>), w: Vec2) -> f32 {
    carpet
        .as_ref()
        .map(|c| c.at(w))
        .or_else(|| spread.as_ref().map(|s| s.at(w)))
        .unwrap_or(0.0)
}
