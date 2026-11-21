//! Lathes: stones stood on rings, such as a basalt column or an obsidian shard. Corner k of every ring
//! sits at the same angle and the same share of its ring's radius, and one shear tilts and leans the
//! whole solid, so every wall and every cap stays flat.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{Facet, hash, lay};
use crate::world::shapes::FlatMesh;

/// One stone stood on rings.
pub(super) struct Lathe {
    /// Where its axis meets the ground, in metres in the frame of the prop.
    pub(super) base: Vec3,
    /// Corners of each ring.
    pub(super) sides: usize,
    /// Height and radius of each ring from the bottom, in metres. The bottom ring is closed by a fan to
    /// the axis, the top ring by a fan to the apex.
    pub(super) rings: Vec<[f32; 2]>,
    /// Height of the apex, in metres: the height of the top ring for a flat top.
    pub(super) apex: f32,
    /// How far the stone leans per metre up, along x and z.
    pub(super) lean: Vec2,
    /// Rise of the stone per metre out from its axis along x and z: a top cut aslant.
    pub(super) tilt: Vec2,
    /// Share of the radius a corner moves in or out, and of the angle step it turns.
    pub(super) wobble: f32,
    /// Seed of the corner moves and of the turn of the first corner.
    pub(super) seed: u32,
}

impl Lathe {
    /// Returns where the point at height `y` and offset `xz` from the axis lands: tilted, then leaned, then
    /// moved onto the base. The map is affine, so flat faces stay flat.
    fn shear(&self, xz: Vec2, y: f32) -> Vec3 {
        let y = y + self.tilt.dot(xz);
        self.base + Vec3::new(xz.x + self.lean.x * y, y, xz.y + self.lean.y * y)
    }

    /// Returns the corners of the ring at height `y` and radius `radius`.
    fn ring(&self, [y, radius]: [f32; 2]) -> Vec<Vec3> {
        let step = TAU / self.sides as f32;
        let turn = hash(self.seed, 0) * step;
        let corner = |k: u32| {
            let swing = self.wobble * (hash(self.seed, 2 * k + 1) - 0.5);
            let out = 1.0 + self.wobble * (2.0 * hash(self.seed, 2 * k + 2) - 1.0);
            let a = turn + (k as f32 + swing) * step;
            self.shear(Vec2::new(a.cos(), a.sin()) * radius * out, y)
        };
        (0..self.sides as u32).map(corner).collect()
    }

    /// Adds the stone to `m`, each face painted by `paint`: the bottom, the walls ring by ring and the top.
    pub(super) fn add(&self, m: &mut FlatMesh, paint: &dyn Fn(&Facet) -> [f32; 4]) {
        let rings: Vec<Vec<Vec3>> = self.rings.iter().map(|r| self.ring(*r)).collect();
        let bottom = self.shear(Vec2::ZERO, self.rings[0][0]);
        let top = self.shear(Vec2::ZERO, self.apex);
        let (n, last) = (self.sides, rings.len() - 1);
        for k in 0..n {
            let j = (k + 1) % n;
            lay(m, [rings[0][k], rings[0][j], bottom], (0, k), paint);
            for (i, w) in rings.windows(2).enumerate() {
                lay(m, [w[0][j], w[0][k], w[1][k]], (i + 1, k), paint);
                lay(m, [w[0][j], w[1][k], w[1][j]], (i + 1, k), paint);
            }
            lay(
                m,
                [rings[last][j], rings[last][k], top],
                (last + 1, k),
                paint,
            );
        }
    }
}
