//! The low architectures of dry ground besides the rosette: mat and vine, each shaped by its species and its
//! age, and the small pieces the low plants share.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::bits::{bead, spike};
use super::kit::{Kit, OLD};
use crate::world::solids::{fan, icosa, prism};

/// Returns the level unit direction at `angle` radians round the vertical.
pub(super) fn level(angle: f32) -> Vec3 {
    Vec3::new(angle.cos(), 0.0, angle.sin())
}

/// Adds `count` spines in a ring round `centre`, each `length` long, pointing out at `rise` over the level,
/// `reach` from the centre and turned by `yaw`.
pub(super) fn ring(
    k: &mut Kit,
    centre: Vec3,
    count: u32,
    (rise, reach, yaw): (f32, f32, f32),
    length: f32,
) {
    let spine = k.tone(k.dials.paint.stem);
    for b in 0..count {
        let out = level(yaw + b as f32 / count as f32 * TAU) + Vec3::Y * rise;
        spike(&mut k.mesh, centre + out * reach, out, length, spine);
    }
}

/// Returns the colour `c` a fifth darker, for the deeper parts of a plant.
pub(super) fn deeper(c: [f32; 4]) -> [f32; 4] {
    [c[0] * 0.8, c[1] * 0.8, c[2] * 0.8, 1.0]
}

/// Adds the cushions of a mat `wide` metres across in `colour`: one when young, two for a many leaved species,
/// scattered over the film or set in a ring round it when old.
fn cushions(k: &mut Kit, wide: f32, colour: [f32; 4]) {
    let (kind, grow) = (k.dials.kind, k.grow);
    let count = 1 + u32::from(k.age > 0 && kind.leaves > 5);
    let radii = Vec3::new(0.32 * grow, 0.07 + 0.08 * (1.0 - kind.droop), 0.3 * grow);
    for c in 0..count {
        let a = c as f32 * 2.7 + k.rand(c + 5) * 0.8;
        let off = [0.35 * k.rand(c + 7).abs(), 0.75][usize::from(k.age == OLD)];
        let at = level(a) * wide * off + Vec3::Y * 0.02;
        icosa(&mut k.mesh, at, radii, (0.2, c), colour);
    }
}

/// Builds a mat: a low film as wide as its species spreads, cushions on it, upright tufts as many as its
/// shoots, and flowers among them.
pub fn mat(k: &mut Kit) {
    let (kind, colour) = (k.dials.kind, k.leaves());
    let wide = (0.55 + 0.3 * kind.spread) * k.grow;
    prism(
        &mut k.mesh,
        Vec3::Y * -0.08,
        Vec2::new(wide, 0.12),
        5,
        colour,
    );
    cushions(k, wide, deeper(colour));
    let tuft = k.tone(k.dials.paint.stem);
    for t in 0..u32::from(kind.branches.saturating_sub(2)) {
        let at = level(t as f32 * 2.1 + k.rand(t + 11)) * wide * 0.5 + Vec3::Y * 0.1;
        bead(&mut k.mesh, at, Vec3::new(0.04, 0.1, 0.04), tuft);
    }
    let spots = [Vec3::new(0.2, 0.12, 0.1), Vec3::new(-0.4, 0.1, 0.25)].map(|p| p * wide);
    k.dress(&spots, 0.06);
}

/// Adds creeper number `s` of `stems` of a vine climbing `climb` metres, with a leaf fan of `blade` at each
/// node and a thorn when armed, and returns where it ends.
fn creeper(k: &mut Kit, (s, stems): (u32, u32), climb: f32, blade: [f32; 5]) -> Vec3 {
    let colour = k.leaves();
    let a = s as f32 / stems as f32 * TAU + k.rand(s + 1) * 0.8;
    let (out, side) = (level(a), level(a + TAU / 4.0));
    let far = (0.8 + 0.3 * k.dials.kind.spread) * k.grow;
    let nodes = [
        Vec3::new(0.0, -0.05, 0.0),
        out * far * 0.5 + Vec3::Y * (climb + 0.1 * k.rand(s + 4)),
        out * far + side * 0.3 * k.rand(s + 8) + Vec3::Y * climb * 0.6,
    ];
    k.stem(nodes[0], nodes[1], [0.05, 0.04]);
    k.stem(nodes[1], nodes[2], [0.04, 0.03]);
    if k.dials.form.thorns {
        spike(&mut k.mesh, nodes[1], Vec3::Y + out, 0.12, colour);
    }
    if k.dials.paint.leaves.is_some() {
        fan(&mut k.mesh, nodes[1] + Vec3::Y * 0.03, blade, a, colour);
        fan(
            &mut k.mesh,
            nodes[2] + Vec3::Y * 0.03,
            blade,
            a + 1.3,
            colour,
        );
    }
    nodes[2] + Vec3::Y * 0.1
}

/// Builds a vine: creepers that curl out from one root, one when young and as many as its shoots later,
/// sprawling low or arching high as its species droops, and a flower or fruit at the end of the first.
pub fn vine(k: &mut Kit) {
    let (kind, size) = (k.dials.kind, k.dials.form.leaf.min(1.4));
    let stems = [1, u32::from(kind.branches.clamp(2, 3))][usize::from(k.age > 0)];
    let climb = (0.25 + 0.5 * (1.0 - kind.droop)) * k.grow;
    let blade = [3.0, 0.3 * size + 0.1, 0.06, 0.0, 0.25 * size + 0.05];
    let ends: Vec<Vec3> = (0..stems)
        .map(|s| creeper(k, (s, stems), climb, blade))
        .collect();
    k.dress(&ends[..1], 0.07);
}
