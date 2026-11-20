//! Coarse views: the terrain as a reader with wide gaps between its samples sees it, which leaves out what
//! such a reader cannot show and what it would cost to find.

use bevy::prelude::*;

use super::Terrain;
use crate::climate::Spot;
use crate::world::Biome;
use crate::world::globe::{Chart, Grain};

impl Terrain {
    /// Returns how closely this view reads the globe.
    pub fn grain(&self) -> Grain {
        self.grain
    }

    /// Returns this view for a reader whose samples lie `spacing` metres apart: it reads the heights, the
    /// biomes and the channel beds without what such a reader is too coarse to show, and without its cost.
    pub fn spaced(&self, spacing: f32) -> Terrain {
        Terrain {
            grain: Grain::of(spacing),
            ..self.clone()
        }
    }

    /// Returns the biomes at three chart positions close together, as this view reads them.
    pub fn biomes(&self, at: [Vec2; 3]) -> [Biome; 3] {
        match self.chart {
            Chart::Flat => at.map(|p| self.globe.biome_flat(p.x, p.y)),
            chart => {
                let dirs = at.map(|p| chart.dir(p.x, p.y));
                self.globe.biomes_seen(dirs, self.grain)
            }
        }
    }

    /// Returns the biomes at three chart positions close together as [`Terrain::biomes`] does, with the
    /// place at the first already read as `spot`, so that a coarse reader samples the ground there once.
    pub fn biomes_glanced(&self, at: [Vec2; 3], spot: Spot) -> [Biome; 3] {
        match self.chart {
            Chart::Flat => self.biomes(at),
            chart => {
                let dirs = at.map(|p| chart.dir(p.x, p.y));
                self.globe.biomes_glanced(dirs, self.grain, || spot)
            }
        }
    }
}
