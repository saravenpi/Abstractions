//! Grain: how far apart a reader samples the ground, which says what it is too coarse to see.
//!
//! A reader whose samples lie farther apart than a river is wide pays for no flow map and no pond, and one
//! whose samples lie farther apart than the nodes of a region grid pays for no grid and no crater either:
//! it reads the bare shape of the land, and its climate from three ground samples.

use bevy::math::{DVec3, Vec2};

use super::Globe;
use super::climate::{INLAND, frame};
use super::season::MEAN_SEASON;
use crate::climate::{self, Spot};
use crate::world::SEA_LEVEL;
use crate::world::biome::Biome;

/// Widest channel in metres: samples farther apart cannot show a river, nor a pond, which is no wider.
const CHANNEL: f32 = 24.0;

/// Metres between two nodes of a region grid at the widest: samples farther apart gain nothing from a grid,
/// and step over every crater.
const REGION: f32 = 125.0;

/// How closely a reader looks at the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grain {
    /// Everything: flows, ponds and craters.
    #[default]
    Fine,
    /// Too coarse for a river or a pond: the ground with its craters, and biomes from the region grids.
    Mid,
    /// Too coarse for a region grid: the bare shape of the land, and biomes from three ground samples.
    Far,
}

impl Grain {
    /// Returns the grain of a reader whose samples lie `spacing` metres apart.
    pub fn of(spacing: f32) -> Self {
        if spacing > REGION {
            Grain::Far
        } else if spacing > CHANNEL {
            Grain::Mid
        } else {
            Grain::Fine
        }
    }
}

impl Globe {
    /// Returns the ground height in metres above sea level in unit direction `dir` as a reader of `grain`
    /// sees it: all of it, or without what the reader is too coarse to show.
    pub fn height_seen(&self, dir: DVec3, grain: Grain) -> f32 {
        let p = dir * self.radius();
        match grain {
            Grain::Fine => self.ground(p),
            _ if !self.has_ground() => SEA_LEVEL,
            Grain::Mid => self.shaped(p),
            Grain::Far => self.formed(p, false).0,
        }
    }

    /// Returns the bare ground height in unit direction `dir` and the place there as the climate reads it,
    /// from three ground samples and no region grid: the height, the inland measure and the rise over the
    /// width of a grid step towards the east and the north, the measures [`Globe::spot`] reads from a grid.
    pub fn glance(&self, dir: DVec3) -> (f32, Spot) {
        let (east, north) = frame(dir);
        let reach = f64::from(REGION) / self.radius();
        let bare = |to: DVec3| self.formed(to.normalize() * self.radius(), false);
        let (height, base) = bare(dir);
        let rise = |along: DVec3| (bare(dir + along * reach).0 - height) / REGION;
        let altitude = match self.has_sea() {
            true => height.max(0.0),
            false => height,
        };
        let spot = Spot {
            dir,
            altitude,
            inland: base / INLAND,
            slope: Vec2::new(rise(east), rise(north)),
        };
        (height, spot)
    }

    /// Returns the biomes in the three unit directions `at`, close together, as a reader of `grain` sees
    /// them. A reader too coarse for a region grid takes the climate of the first for all three, each
    /// border blurred by its own noise.
    pub fn biomes_seen(&self, at: [DVec3; 3], grain: Grain) -> [Biome; 3] {
        if grain != Grain::Far || self.lands().biome().is_some() {
            return at.map(|dir| self.biome(dir));
        }
        let climate = climate::at(self.setting(), &self.glance(at[0]).1, MEAN_SEASON);
        at.map(|dir| climate::biome(&climate, self.blur(dir * self.radius())))
    }
}
