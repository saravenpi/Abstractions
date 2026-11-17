//! Terrain level of detail: grid strides per ring of chunks around the focus, with hysteresis.

use bevy::prelude::*;

#[cfg(test)]
use super::rings::Rings;

/// How coarse the terrain grid of a chunk is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lod {
    /// The full 2 m grid.
    Near,
    /// Every second grid point, a 4 m grid.
    Mid,
    /// Every fourth grid point, an 8 m grid.
    Far,
    /// The 8 m grid with a flat sea drawn in, for chunks the near sea does not reach.
    Distant,
}

impl Lod {
    /// Returns how many full-resolution grid steps lie between two vertices.
    pub fn stride(self) -> usize {
        match self {
            Lod::Near => 1,
            Lod::Mid => 2,
            Lod::Far | Lod::Distant => 4,
        }
    }

    /// Returns the level wanted for a chunk `ring` chunks away from the focus at the default
    /// render distance.
    #[cfg(test)]
    pub fn for_ring(ring: i32) -> Lod {
        Rings::DEFAULT.lod(ring)
    }
}

/// Returns the level a chunk at `ring` should use now at the default render distance: finer levels
/// apply at once, coarser ones one ring later.
#[cfg(test)]
pub fn next_lod(current: Lod, ring: i32) -> Lod {
    Rings::DEFAULT.next(current, ring)
}

/// Returns the Chebyshev distance in chunks between `coord` and `center`.
pub fn ring_distance(coord: IVec2, center: IVec2) -> i32 {
    (coord - center).abs().max_element()
}

/// Keeps every `stride`-th point of a square height grid along both axes.
pub fn subsample(heights: &[Vec<f32>], stride: usize) -> Vec<Vec<f32>> {
    heights
        .iter()
        .step_by(stride)
        .map(|row| row.iter().step_by(stride).copied().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Levels follow the rings and only coarsen one ring later than they refine.
    #[test]
    fn levels_have_hysteresis() {
        assert_eq!(Lod::for_ring(1), Lod::Near);
        assert_eq!(Lod::for_ring(2), Lod::Mid);
        assert_eq!(Lod::for_ring(4), Lod::Far);
        assert_eq!(next_lod(Lod::Near, 2), Lod::Near);
        assert_eq!(next_lod(Lod::Near, 3), Lod::Mid);
        assert_eq!(next_lod(Lod::Mid, 2), Lod::Mid);
        assert_eq!(next_lod(Lod::Mid, 1), Lod::Near);
        assert_eq!(next_lod(Lod::Mid, 4), Lod::Mid);
        assert_eq!(next_lod(Lod::Mid, 5), Lod::Far);
        assert_eq!(next_lod(Lod::Far, 3), Lod::Mid);
    }

    /// A subsampled 33 point grid keeps 17 and 9 points per side.
    #[test]
    fn subsample_size() {
        let grid = vec![vec![0.0; 33]; 33];
        assert_eq!(subsample(&grid, 2).len(), 17);
        assert_eq!(subsample(&grid, 4)[0].len(), 9);
    }
}
