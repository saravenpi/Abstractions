//! The merged ground-cover mesh: grass, flowers, seagrass, kelp, and pond plants and small solids such as coral.

use bevy::prelude::*;

use crate::world::propmesh::{BIG, FLOWER, Plant, solid_tris};
use crate::world::scatter::{Placement, Scatter};
use crate::world::shapes::{FlatMesh, shade};
use crate::world::solids::{Fan, fan};

use super::kelp::{kelp, kelp_tris};
use super::pond::{pond, pond_tris};

/// One group of blades: fan shape, lift above the ground and sRGB color.
type Layer = (Fan, f32, [f32; 3]);

/// Sea green of the kelp ribbons.
const KELP: [f32; 3] = [0.36, 0.42, 0.14];

/// Blade layers of every fan plant, in [`Plant`] order from grass to shell.
#[rustfmt::skip]
const COVER: [&[Layer]; 10] = [
    &[([3.0, 0.12, 0.55, 0.0, 0.16], 0.0, [0.30, 0.58, 0.22])],
    &[([4.0, 0.1, 1.3, 0.0, 0.09], 0.0, [0.45, 0.6, 0.3]), ([3.0, 0.02, 0.25, 0.0, 0.08], 1.3, [0.45, 0.3, 0.15])],
    &[([5.0, 0.7, 0.2, 0.55, 0.5], 0.0, [0.12, 0.5, 0.2])],
    &[([5.0, 0.45, 0.2, 0.35, 0.22], 0.0, [0.2, 0.5, 0.2])],
    &[([2.0, 0.0, 0.18, 0.0, 0.07], 0.0, [0.9, 0.85, 0.75]), ([5.0, 0.11, -0.04, 0.0, 0.12], 0.18, [0.8, 0.12, 0.1])],
    &[([4.0, 0.2, 1.1, 0.1, 0.16], 0.0, [0.66, 0.68, 0.3])],
    &[([5.0, 0.35, 0.0, 0.0, 0.25], 0.04, [0.78, 0.4, 0.12]), ([4.0, 0.3, 0.0, 0.0, 0.2], 0.05, [0.6, 0.25, 0.1])],
    &[([5.0, 0.15, 1.0, 0.0, 0.08], 0.0, [0.16, 0.55, 0.34])],
    &[([5.0, 0.28, 0.03, 0.0, 0.13], 0.03, [0.96, 0.45, 0.2]), ([5.0, 0.1, 0.03, 0.0, 0.1], 0.05, [0.98, 0.85, 0.5])],
    &[([7.0, 0.16, 0.07, 0.0, 0.1], 0.03, [0.96, 0.8, 0.78])],
];

/// Petal and center colors of the flowers, in [`Plant`] order.
#[rustfmt::skip]
const HEADS: [([f32; 3], [f32; 3]); 6] = [
    ([0.88, 0.12, 0.1], [0.1, 0.07, 0.07]),
    ([0.97, 0.97, 0.95], [0.97, 0.8, 0.15]),
    ([0.98, 0.82, 0.12], [0.9, 0.55, 0.1]),
    ([0.35, 0.45, 0.95], [0.9, 0.9, 0.5]),
    ([0.97, 0.55, 0.7], [0.98, 0.85, 0.3]),
    ([0.9, 0.25, 0.7], [1.0, 0.75, 0.2]),
];

/// Returns the blade groups of a fan plant.
fn layers(plant: Plant) -> Vec<Layer> {
    let i = plant as usize;
    if i < FLOWER {
        return COVER[i - BIG].to_vec();
    }
    let (petal, heart) = HEADS[i - FLOWER];
    vec![
        ([2.0, 0.0, 0.4, 0.0, 0.05], 0.0, [0.25, 0.5, 0.2]),
        ([4.0, 0.1, 0.0, 0.0, 0.1], 0.4, petal),
        ([3.0, 0.03, 0.02, 0.0, 0.06], 0.41, heart),
    ]
}

/// Returns how many triangles a small plant adds to the ground-cover mesh.
pub fn small_tris(p: &Placement) -> usize {
    if p.plant == Plant::Kelp {
        kelp_tris(p.scale)
    } else if p.plant.pond() {
        pond_tris(p.plant)
    } else if p.plant.solid() {
        solid_tris(p.plant as usize)
    } else {
        let blades = |l: &Layer| l.0[0] as usize * if l.0[3] == 0.0 { 2 } else { 6 };
        layers(p.plant).iter().map(blades).sum()
    }
}

/// Adds small plant `p`, number `i` of its chunk, to `m` in chunk space. Small solids add nothing: they are merged
/// from shared geometry.
pub fn sprout(m: &mut FlatMesh, i: usize, p: &Placement) {
    let tone = 0.9 + (i % 5) as f32 * 0.05;
    if p.plant == Plant::Kelp {
        kelp(m, p.pos, p.scale, p.yaw, shade(KELP, tone));
    } else if p.plant.pond() {
        pond(m, p, tone);
    } else if !p.plant.solid() {
        for (spec, lift, color) in layers(p.plant) {
            let [count, reach, rise, arch, width] = spec;
            let s = p.scale;
            let shape = [count, reach * s, rise * s, arch * s, width * s];
            fan(
                m,
                p.pos + Vec3::Y * lift * s,
                shape,
                p.yaw,
                shade(color, tone),
            );
        }
    }
}

/// Returns the small solids of a scatter, such as coral and mushrooms.
pub fn solids(scatter: &Scatter) -> Vec<Placement> {
    let solid = |p: &&Placement| p.plant.solid();
    scatter.grass.iter().filter(solid).copied().collect()
}
