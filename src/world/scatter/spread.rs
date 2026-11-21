//! The turf of a stretch of ground wider than a chunk, read on a small lattice: the same cover the chunks hold,
//! so a coarse mesh shows the carpet its fine chunks grow, at the cost of a few reads per mesh and not one per cell.

use bevy::prelude::*;

use super::air::still;
use super::grow::Flora;
use super::sampler::Sampler;
use super::turf::turf_of;
use crate::world::Terrain;
use crate::world::lands::thrive::climate_at;

/// How thick the turf grows over a square of the chart, read at the points of a lattice and eased between.
pub struct Spread {
    /// Thickness at each lattice point, the first index along x and the second along z.
    grid: Vec<Vec<f32>>,
    /// Chart position of the low corner of the square.
    corner: Vec2,
    /// Side of the square in chart units.
    size: f32,
}

impl Spread {
    /// Returns the turf over the square from chart position `lo` to `hi` of `terrain`, read at `side` by `side`
    /// points in the annual mean climate of the place, or nothing where no ground cover grows there.
    pub fn over(terrain: &Terrain, (lo, hi): (Vec2, Vec2), side: usize) -> Option<Self> {
        let flora = Flora::of(terrain, (lo + hi) * 0.5)?;
        let density = turf_of(&flora)?.density.sqrt();
        let sampler = Sampler::new(terrain, flora.patchiness);
        let last = (side - 1) as f32;
        let read = |i: usize, j: usize| {
            let w = lo + (hi - lo) * Vec2::new(i as f32, j as f32) / last;
            let wet = match still() {
                true => 0.5,
                false => climate_at(terrain, w).map_or(0.5, |c| c.moisture.min(1.2)),
            };
            sampler.at(terrain, w, wet)[0] * density
        };
        Some(Self {
            grid: (0..side)
                .map(|i| (0..side).map(|j| read(i, j)).collect())
                .collect(),
            corner: lo,
            size: hi.x - lo.x,
        })
    }

    /// Returns how thick the turf grows at chart position `w`, 0 (barren) to 1, eased between the lattice points.
    pub fn at(&self, w: Vec2) -> f32 {
        let last = self.grid.len() - 1;
        let top = last as f32 - 1e-3;
        let t = ((w - self.corner) / self.size * last as f32).clamp(Vec2::ZERO, Vec2::splat(top));
        let (i, j, f) = (t.x as usize, t.y as usize, t.fract());
        let low = self.grid[i][j] + (self.grid[i + 1][j] - self.grid[i][j]) * f.x;
        let high = self.grid[i][j + 1] + (self.grid[i + 1][j + 1] - self.grid[i][j + 1]) * f.x;
        low + (high - low) * f.y
    }
}
