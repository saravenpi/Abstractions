//! Patch noise and depth windows that decide how thick each kind of plant grows at a spot.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use super::spacing::stone;
use crate::world::Terrain;
use crate::world::globe::Chart;
use crate::world::propmesh::{BIG, FLOWER, Plant, TREES};
use crate::world::rocks::tune::LODE_FREQ;

/// Shallowest water in which kelp grows.
const KELP_TOP: f32 = 5.5;

/// Deepest water in which kelp grows.
const KELP_BOTTOM: f32 = 21.0;

/// How thick each family of plants grows at one spot, as multipliers on the table chances.
pub struct Density {
    /// Tree groves.
    pub grove: f32,
    /// Flower blooms.
    pub bloom: f32,
    /// Coral, seagrass and shell patches, thicker in shallow water.
    pub patch: f32,
    /// Kelp forests, zero outside their depth window.
    pub forest: f32,
    /// Reeds and cattails, which only grow where the water is shallower than half a meter.
    pub edge: f32,
    /// Share of the land plants the climate lets grow, 1 where it is not read.
    pub vigor: f32,
    /// True above the snow line of the climate, where only stones lie.
    pub frozen: bool,
}

/// A place where the density fields can be read.
///
/// A bare `Vec2` is a position of the legacy plane. A terrain view with a chart position reads
/// the fields at the 3D point of the globe, so the density shows no seam between cube faces.
pub trait Spot: Copy {
    /// Returns the value of `field` at this place, `freq` being its frequency per metre.
    fn read(self, field: &Perlin, freq: f64) -> f64;
}

impl Spot for Vec2 {
    fn read(self, field: &Perlin, freq: f64) -> f64 {
        field.get([self.x as f64 * freq, self.y as f64 * freq])
    }
}

impl Spot for (&Terrain, Vec2) {
    fn read(self, field: &Perlin, freq: f64) -> f64 {
        let (terrain, w) = self;
        if terrain.chart() == Chart::Flat {
            return w.read(field, freq);
        }
        field.get((terrain.embed(w.x, 0.0, w.y) * freq).to_array())
    }
}

/// The low-frequency noise fields behind [`Density`], and the lode that picks the stone of a field of rocks.
pub struct Fields([Perlin; 5]);

impl Fields {
    /// Builds the fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self([0x51, 0xA3, 0x7B, 0xC9, 0xE7].map(|salt| Perlin::new(seed as u32 ^ salt)))
    }

    /// Returns the lode at `w`: a slow field from 0 to 1 that picks which hard stone a field of rocks is
    /// made of, so a scree is mostly one stone.
    pub fn lode(&self, w: impl Spot) -> f32 {
        (w.read(&self.0[4], LODE_FREQ) as f32 * 0.9 + 0.5).clamp(0.0, 1.0)
    }

    /// Returns the densities at `w` over a floor `depth` meters below the sea, negative on land.
    ///
    /// `w` is a world position of the legacy plane, or a terrain view with a chart position.
    pub fn density(&self, w: impl Spot, depth: f32) -> Density {
        let at = |i: usize, freq: f64| w.read(&self.0[i], freq) as f32;
        let soft = |n: f32| (n * 2.0 + 0.5).clamp(0.0, 1.0);
        let shore = (1.0 - depth / 6.0).clamp(0.0, 1.0);
        let window = ((depth - KELP_TOP) / 1.5).clamp(0.0, 1.0)
            * ((KELP_BOTTOM - depth) / 3.0).clamp(0.0, 1.0);
        let edge = if depth > 0.0 {
            ((0.5 - depth) / 0.15).clamp(0.0, 1.0)
        } else {
            1.0
        };
        Density {
            edge,
            vigor: 1.0,
            frozen: false,
            grove: soft(at(0, 0.045)),
            bloom: soft(at(1, 0.07)),
            patch: (at(2, 0.06) * 4.0 + 0.2).clamp(0.0, 1.6) * (0.6 + 0.8 * shore),
            forest: (at(3, 0.03) * 4.0 + 0.4).clamp(0.0, 1.6) * window,
        }
    }
}

/// Returns the multiplier a plant's table chance gets from the local density, and from the climate
/// unless it is a stone.
fn gain(plant: Plant, d: &Density) -> f32 {
    let i = plant as usize;
    let grow = [d.vigor, 1.0][usize::from(stone(plant))];
    grow * if matches!(plant, Plant::Reed | Plant::Cattail) {
        d.edge
    } else if plant == Plant::Kelp {
        d.forest
    } else if plant.patchy() {
        d.patch
    } else if i < TREES {
        d.grove
    } else if i >= FLOWER {
        d.bloom
    } else {
        1.0
    }
}

/// Picks the plant a roll in `[0, 1)` lands on among the big or the small plants of a table.
pub fn choose(table: &[(Plant, f32)], roll: f32, d: &Density, big: bool) -> Option<Plant> {
    let mut acc = 0.0;
    let kept = |p: &Plant| ((*p as usize) < BIG) == big && (!d.frozen || stone(*p));
    for &(plant, chance) in table.iter().filter(|(p, _)| kept(p)) {
        acc += chance * gain(plant, d);
        if roll < acc {
            return Some(plant);
        }
    }
    None
}
