//! Patch noise and depth windows that decide how thick each kind of plant grows at a spot.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::world::propmesh::{BIG, FLOWER, Plant, TREES};

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
}

/// The low-frequency noise fields behind [`Density`].
pub struct Fields([Perlin; 4]);

impl Fields {
    /// Builds the fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self([0x51, 0xA3, 0x7B, 0xC9].map(|salt| Perlin::new(seed as u32 ^ salt)))
    }

    /// Returns the densities at world position `w` over a floor `depth` meters below the sea, negative on land.
    pub fn density(&self, w: Vec2, depth: f32) -> Density {
        let at = |i: usize, freq: f64| self.0[i].get([w.x as f64 * freq, w.y as f64 * freq]) as f32;
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
            grove: soft(at(0, 0.045)),
            bloom: soft(at(1, 0.07)),
            patch: (at(2, 0.06) * 4.0 + 0.2).clamp(0.0, 1.6) * (0.6 + 0.8 * shore),
            forest: (at(3, 0.03) * 4.0 + 0.4).clamp(0.0, 1.6) * window,
        }
    }
}

/// Returns the multiplier a plant's table chance gets from the local density.
fn gain(plant: Plant, d: &Density) -> f32 {
    let i = plant as usize;
    if matches!(plant, Plant::Reed | Plant::Cattail) {
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
    for &(plant, chance) in table.iter().filter(|(p, _)| ((*p as usize) < BIG) == big) {
        acc += chance * gain(plant, d);
        if roll < acc {
            return Some(plant);
        }
    }
    None
}
