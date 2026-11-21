//! Blobs: rounded stones built from an icosphere. Each corner is pushed in or out by a hash of its
//! direction, pressed flat onto a few cut planes, then stretched to the radii of the stone, so a cut
//! stays a flat facet with a sharp rim.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{Facet, hash, lay, span};
use crate::world::shapes::FlatMesh;
use crate::world::solids::{FACES, icosa_vertices};

/// Distance within which a corner counts as lying on a cut plane, in unit-sphere lengths.
const ON: f32 = 1.0e-4;

/// Steps per unit length at which a direction is rounded before its corner is hashed.
const GRID: f32 = 4096.0;

/// One rounded stone.
pub(super) struct Blob {
    /// Middle of the stone, in metres in the frame of the prop.
    pub(super) centre: Vec3,
    /// Half-sizes along x, y and z, in metres.
    pub(super) radii: Vec3,
    /// How many times the icosahedron is split: 0 gives 20 faces, 1 gives 80.
    pub(super) level: u32,
    /// Share of the radius a corner moves in or out.
    pub(super) wobble: f32,
    /// Seed of the corner moves.
    pub(super) seed: u32,
    /// Planes the stone is cut flat on: an outward unit normal and the distance from the middle, in
    /// unit-sphere lengths. A cut further than half the radius out keeps every face turned outward.
    pub(super) cuts: Vec<(Vec3, f32)>,
}

/// Returns `count` cut planes drawn from `seed`: spread around the stone, rising at most `rise` toward the
/// top, each between the two ends of `depth` from the middle.
pub(super) fn cuts(seed: u32, count: usize, depth: [f32; 2], rise: f32) -> Vec<(Vec3, f32)> {
    let turn = span(seed, 19, [0.0, TAU]);
    (0..count as u32)
        .map(|k| {
            let a = turn + k as f32 * 2.4 + span(seed, 20 + 3 * k, [-0.5, 0.5]);
            let up = span(seed, 21 + 3 * k, [-0.2, rise]);
            let normal = Vec3::new(a.cos(), up, a.sin()).normalize();
            (normal, span(seed, 22 + 3 * k, depth))
        })
        .collect()
}

/// Splits a face into four, its new corners pushed out onto the unit sphere. The middle of an edge is
/// the same point from both faces that share it.
fn split([a, b, c]: [Vec3; 3]) -> [[Vec3; 3]; 4] {
    let mid = |p: Vec3, q: Vec3| (p + q).normalize();
    let (ab, bc, ca) = (mid(a, b), mid(b, c), mid(c, a));
    [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]
}

/// Returns the faces of a unit icosphere split `level` times, wound outward.
fn sphere(level: u32) -> Vec<[Vec3; 3]> {
    let v = icosa_vertices();
    let mut faces: Vec<[Vec3; 3]> = FACES.map(|f| f.map(|i| v[i])).to_vec();
    for _ in 0..level {
        faces = faces.into_iter().flat_map(split).collect();
    }
    faces
}

impl Blob {
    /// Returns where unit direction `dir` lands on the stone before the stretch: moved in or out by a hash
    /// of the direction, so a corner that several faces share lands on one point, then pressed onto every
    /// cut it lies past, twice over so that two cuts meet cleanly.
    fn press(&self, dir: Vec3) -> Vec3 {
        let key = (dir * GRID).round().as_ivec3();
        let salt = (key.x as u32).wrapping_mul(73_856_093)
            ^ (key.y as u32).wrapping_mul(19_349_663)
            ^ (key.z as u32).wrapping_mul(83_492_791);
        let mut p = dir * (1.0 + self.wobble * (2.0 * hash(self.seed, salt) - 1.0));
        for &(normal, depth) in self.cuts.iter().chain(&self.cuts) {
            p -= normal * (p.dot(normal) - depth).max(0.0);
        }
        p
    }

    /// Returns the band of a face whose corners lie at `unit` before the stretch: one more than the cut all
    /// three lie on, or 0 on the rind.
    fn band(&self, unit: &[Vec3; 3]) -> usize {
        let on = |cut: &(Vec3, f32)| unit.iter().all(|p| p.dot(cut.0) >= cut.1 - ON);
        self.cuts.iter().position(on).map_or(0, |k| k + 1)
    }

    /// Adds the stone to `m`, each face painted by `paint`.
    pub(super) fn add(&self, m: &mut FlatMesh, paint: &dyn Fn(&Facet) -> [f32; 4]) {
        for (number, face) in sphere(self.level).into_iter().enumerate() {
            let unit = face.map(|d| self.press(d));
            let corners = unit.map(|p| self.centre + p * self.radii);
            lay(m, corners, (self.band(&unit), number), paint);
        }
    }
}
