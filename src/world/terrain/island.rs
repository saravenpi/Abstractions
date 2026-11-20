//! Whether land surrounds a spawn: the rings of heights read around it.

use bevy::prelude::*;

use super::Terrain;
use super::spawn::{ISLAND_RADII, ISLAND_SAMPLES, ISLAND_SHARE};

impl Terrain {
    /// Returns the heights sampled on a ring of radius `r` around `(x, z)`.
    pub(super) fn ring_heights(&self, x: f32, z: f32, r: f32) -> impl Iterator<Item = f32> {
        (0..ISLAND_SAMPLES).map(move |a| {
            let d = Vec2::from_angle(a as f32 * std::f32::consts::TAU / ISLAND_SAMPLES as f32) * r;
            self.height(x + d.x, z + d.y)
        })
    }

    /// Returns true when land surrounds `(x, z)` out to about two hectares.
    pub(super) fn on_island(&self, x: f32, z: f32) -> bool {
        let total = (ISLAND_RADII.len() as u32 * ISLAND_SAMPLES) as f32;
        let sea = self.globe().sea_level();
        let dry = ISLAND_RADII
            .iter()
            .flat_map(|&r| self.ring_heights(x, z, r))
            .filter(|&h| h > sea)
            .count();
        dry as f32 / total >= ISLAND_SHARE
    }

    /// Returns a penalty in metres for a spawn that is inland.
    pub(super) fn inland_penalty(&self, x: f32, z: f32) -> f32 {
        let sea = self.globe().sea_level();
        200.0 * f32::from(!self.ring_heights(x, z, 70.0).any(|h| h <= sea))
    }
}
