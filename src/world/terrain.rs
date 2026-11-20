//! The terrain height field: the single source of truth for where land is.

use std::sync::Arc;

use bevy::prelude::*;

use super::biome::Biome;
use super::globe::{Chart, Globe, Grain};
use super::surface::Surface;
use crate::universe::{HOME_GRAVITY, HOME_RADIUS};

mod coarse;
mod landfall;
mod ponds;
mod root;
mod spawn;
mod view;

/// A flat `(x, z)` view of the ground of one planet, deterministic for a seed.
///
/// The chart says how `(x, z)` maps onto the globe. Heights are metres above sea level. On a cube face
/// `(x, z)` are 32-bit offsets from the 64-bit origin of the chart, so they stay precise on a planet of
/// any size; whole tiles counted from the middle of the face name ground that outlives a view.
#[derive(Resource, Clone)]
pub struct Terrain {
    globe: Arc<Globe>,
    chart: Chart,
    /// How closely this view reads the globe: everything, unless it was made for a coarse mesh.
    grain: Grain,
}

impl Terrain {
    /// Builds the legacy flat terrain for a seed. The same seed always gives the same world.
    pub fn new(seed: u64) -> Self {
        let globe = Globe::new(seed, HOME_RADIUS, HOME_GRAVITY);
        Self::on(Arc::new(globe), Chart::Flat)
    }

    /// Builds the view of `globe` through `chart`.
    pub fn on(globe: Arc<Globe>, chart: Chart) -> Self {
        let grain = Grain::Fine;
        Self {
            globe,
            chart,
            grain,
        }
    }

    /// Returns the seed this terrain was built from.
    pub fn seed(&self) -> u64 {
        self.globe.seed()
    }

    /// Returns the ground height at chart position `(x, z)`.
    pub fn height(&self, x: f32, z: f32) -> f32 {
        match self.chart {
            Chart::Flat => self.globe.height_flat(x, z),
            chart => self.globe.height_seen(chart.dir(x, z), self.grain),
        }
    }

    /// Returns the height the finest terrain mesh draws at `(x, z)`: its 2 m triangles, interpolated.
    ///
    /// Anything that must touch the visible ground stands on this, not on [`Terrain::height`].
    pub fn mesh_height(&self, x: f32, z: f32) -> f32 {
        let step = super::mesh::GRID_STEP;
        let (i, j) = ((x / step).floor(), (z / step).floor());
        let (u, v) = (x / step - i, z / step - j);
        let h = |a: f32, b: f32| self.height((i + a) * step, (j + b) * step);
        if u + v <= 1.0 {
            h(0.0, 0.0) + u * (h(1.0, 0.0) - h(0.0, 0.0)) + v * (h(0.0, 1.0) - h(0.0, 0.0))
        } else {
            h(1.0, 1.0)
                + (1.0 - u) * (h(0.0, 1.0) - h(1.0, 1.0))
                + (1.0 - v) * (h(1.0, 0.0) - h(1.0, 1.0))
        }
    }

    /// Returns true when `(x, z)` lies inside a pond basin.
    pub fn pond_at(&self, x: f32, z: f32) -> bool {
        match self.chart {
            Chart::Flat => self.globe.pond_flat(x, z),
            chart => self.globe.pond_at(chart.dir(x, z)),
        }
    }

    /// Returns what covers the ground at chart position `(x, z)`.
    pub fn surface(&self, x: f32, z: f32) -> Surface {
        match self.chart {
            Chart::Flat => self.globe.surface_flat(x, z),
            chart => self.globe.surface(chart.dir(x, z)),
        }
    }

    /// Returns the biome at chart position `(x, z)`, also over water.
    pub fn biome(&self, x: f32, z: f32) -> Biome {
        match self.chart {
            Chart::Flat => self.globe.biome_flat(x, z),
            chart => self.globe.biome(chart.dir(x, z)),
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_root;
#[cfg(test)]
mod tests_spawn;
