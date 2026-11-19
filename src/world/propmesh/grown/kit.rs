//! The kit a grown plant is built with: its mesh, its dials, a stable jitter and the dressing every
//! architecture shares.

use bevy::prelude::*;

use super::Look;
use super::bits::{bead, spike};
use crate::life::flora::Dials;
use crate::world::shapes::FlatMesh;
use crate::world::solids::frustum;

/// Linear RGB of resin beads.
const RESIN: [f32; 3] = [0.55, 0.22, 0.01];

/// Linear RGB of thorns.
const THORN: [f32; 3] = [0.5, 0.45, 0.35];

/// A grown plant being built.
pub struct Kit {
    /// The triangles so far, in the frame of the plant: base at the origin, +Y up.
    pub mesh: FlatMesh,
    /// The dials of its group.
    pub dials: Dials,
    /// Seed of its jitter, from its architecture and its variant.
    seed: u32,
}

impl Kit {
    /// Returns an empty kit for `look`.
    pub fn new(look: &Look) -> Self {
        let arch = u32::from(look.dials.architecture);
        Self {
            mesh: FlatMesh::default(),
            dials: look.dials,
            seed: (u32::from(look.variant) + 1).wrapping_mul(0x9E37_79B9)
                ^ arch.wrapping_mul(0x85EB_CA6B),
        }
    }

    /// Returns a stable value in `[-1, 1]` for draw number `i`.
    pub fn rand(&self, i: u32) -> f32 {
        let h = (i ^ self.seed).wrapping_mul(0x85EB_CA6B);
        let h = (h ^ (h >> 13)).wrapping_mul(0xC2B2_AE35);
        ((h >> 8) & 0xFFFF) as f32 / 32767.5 - 1.0
    }

    /// Returns linear colour `c` as the colour lookup takes it, a little darker or lighter by variant.
    pub fn tone(&self, c: [f32; 3]) -> [f32; 4] {
        let k = 1.0 + 0.06 * self.rand(97);
        [c[0] * k, c[1] * k, c[2] * k, 1.0]
    }

    /// Returns the colour of the leaves, or of the stems while the plant stands bare.
    pub fn leaves(&self) -> [f32; 4] {
        let paint = self.dials.paint;
        self.tone(paint.leaves.unwrap_or(paint.stem))
    }

    /// Adds a stem from `from` to `to` with the radii `radii` at both ends, in the colour of the stems.
    pub fn stem(&mut self, from: Vec3, to: Vec3, radii: [f32; 2]) {
        let colour = self.tone(self.dials.paint.stem);
        frustum(&mut self.mesh, from, to, Vec2::from(radii), colour);
    }

    /// Hangs a flower on every one of `spots` and a fruit under the first four, as the paint allows, each of
    /// about `size` metres.
    pub fn dress(&mut self, spots: &[Vec3], size: f32) {
        let paint = self.dials.paint;
        for (i, &at) in spots.iter().enumerate() {
            if let Some(c) = paint.flowers.map(|c| self.tone(c)) {
                bead(&mut self.mesh, at, Vec3::new(1.0, 0.6, 1.0) * size, c);
            }
            if let Some(c) = paint.fruit.filter(|_| i < 4).map(|c| self.tone(c)) {
                bead(
                    &mut self.mesh,
                    at - Vec3::Y * size * 1.6,
                    Vec3::splat(size * 0.8),
                    c,
                );
            }
        }
    }

    /// Adds the visible defences along the stem from `from` to `to` of radius `reach`: four thorns, and two
    /// beads of resin.
    pub fn armour(&mut self, from: Vec3, to: Vec3, reach: f32) {
        let (form, thorn, resin) = (self.dials.form, self.tone(THORN), self.tone(RESIN));
        for k in 0..4 {
            let at = from.lerp(to, 0.2 + 0.18 * k as f32);
            let a = k as f32 * 2.4 + self.rand(k);
            let out = Vec3::new(a.cos(), 0.3, a.sin());
            if form.thorns {
                spike(
                    &mut self.mesh,
                    at + out * reach * 0.8,
                    out,
                    reach * 1.2,
                    thorn,
                );
            }
            if form.resin && k % 2 == 1 {
                bead(
                    &mut self.mesh,
                    at + out * reach,
                    Vec3::splat(reach * 0.45),
                    resin,
                );
            }
        }
    }
}
