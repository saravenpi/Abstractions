//! The tree: a leaning trunk that forks into scaffold branches, each carrying a clump of leaves, cones of
//! needles in the cold, or bare twigs while it is dormant. It starts half a metre under the ground.
//!
//! Its species sets how wide it spreads, how far it leans, how many branches it forks into and how its leaves
//! hang; its age sets its girth, its crown and the scars of an old tree.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::kit::{Kit, OLD};
use crate::world::solids::{cone, icosa};

/// Height in metres of a grown tree as its mesh is built, its reference height.
const TALL: f32 = 4.0;

/// Radius in metres of a scaffold branch where it leaves the trunk: thin enough for a three sided limb.
const ARM: f32 = 0.085;

/// Where a tree stands in its life: the share of its height that is bare trunk, its girth at the foot in
/// metres, its scaffold branches and the size of its crown against a grown one.
struct Frame {
    /// Share of the height that is bare trunk.
    share: f32,
    /// Girth at the foot in metres.
    girth: f32,
    /// Scaffold branches.
    arms: u32,
    /// Size of the crown against a grown tree.
    crown: f32,
}

/// Returns the frame of the tree `k` builds, from its species, its form and its age.
fn frame(k: &Kit) -> Frame {
    let (form, kind) = (k.dials.form, k.dials.kind);
    let girth = if form.store { 0.5 } else { 0.24 };
    let share = match form.needles {
        true => 0.85,
        false => 0.58 - 0.1 * (kind.spread - 1.0),
    };
    let arms = u32::from(kind.branches.clamp(2, 5));
    match k.age {
        0 => Frame {
            share: share + 0.08,
            girth: girth * 0.55,
            arms: 2,
            crown: 0.75,
        },
        OLD => Frame {
            share: share - 0.06,
            girth: girth * 1.35,
            arms: (arms + 1).min(5),
            crown: 1.15,
        },
        _ => Frame {
            share,
            girth,
            arms,
            crown: 1.0,
        },
    }
}

/// Builds a tree: a buried leaning trunk, a buttressed foot and a broken stub when old, then cones of needles
/// in the cold, a crown of leaf clumps on its branches elsewhere, or bare twigs while it is dormant.
pub fn tree(k: &mut Kit) {
    let f = frame(k);
    let (base, up) = (Vec3::new(0.0, -0.5, 0.0), k.axis());
    let top = base + up * (TALL * f.share + 0.5);
    k.stem(base, top, [f.girth, f.girth * 0.55]);
    k.armour(Vec3::ZERO, top, f.girth * 0.7);
    if k.age == OLD {
        k.stem(base, Vec3::Y * 0.35, [f.girth * 1.7, f.girth]);
        let stub = base.lerp(top, 0.55);
        let out = Vec3::new(k.rand(5), 0.4, k.rand(6)).normalize();
        k.stem(stub, stub + out * 0.7, [0.06, 0.02]);
    }
    if k.dials.form.needles {
        conifer(k, (base, up), &f);
        return;
    }
    let tips = scaffold(k, (base, top), &f);
    if k.dials.paint.leaves.is_none() {
        twigs(k, &tips);
        return;
    }
    let spots = canopy(k, top, &tips, f.crown);
    k.dress(&spots, 0.12);
}

/// Adds the scaffold branches of frame `f` forking from the upper part of the trunk from `base` to `top`,
/// spreading as wide and rising as steeply as the species lets them, and returns their tips.
fn scaffold(k: &mut Kit, (base, top): (Vec3, Vec3), f: &Frame) -> Vec<Vec3> {
    let kind = k.dials.kind;
    let reach = (0.6 + 0.65 * kind.spread) * f.crown;
    let rise = 1.25 - 0.5 * kind.spread - 0.25 * kind.droop;
    (0..f.arms)
        .map(|a| {
            let turn = a as f32 / f.arms as f32 * TAU + k.rand(a + 3) * 0.6;
            let from = base.lerp(top, 0.7 + 0.25 * a as f32 / f.arms as f32);
            let out = Vec3::new(turn.cos(), rise, turn.sin()).normalize();
            let tip = from + out * reach * (1.0 + 0.2 * k.rand(a + 9));
            k.stem(from, tip, [ARM, 0.035]);
            tip
        })
        .collect()
}

/// Adds a clump of leaves over each of `tips` and a larger one over the trunk top `top`, as big as `crown`
/// lets them and hanging as far as the species droops, and returns the spots its flowers and fruit hang on.
fn canopy(k: &mut Kit, top: Vec3, tips: &[Vec3], crown: f32) -> Vec<Vec3> {
    let (form, kind, colour) = (k.dials.form, k.dials.kind, k.leaves());
    let dense = 0.85 + 0.06 * f32::from(form.layers.clamp(1, 4));
    let size = (0.5 + 0.22 * form.leaf.min(1.4)) * dense * crown;
    let crest = top + k.axis() * 0.8 * crown;
    let all = tips.iter().copied().chain(std::iter::once(crest));
    let mut spots = Vec::new();
    for (i, at) in all.enumerate() {
        let big = if i == tips.len() { 1.2 } else { 1.0 };
        let r = size * big * (1.0 + 0.18 * k.rand(i as u32 + 30));
        let wide = r * (1.0 - 0.25 * kind.droop);
        let radii = Vec3::new(wide, r * (0.62 + 0.6 * kind.droop), wide);
        let centre = at + Vec3::Y * r * (0.2 - 0.5 * kind.droop);
        let seed = i as u32 * 7 + u32::from(kind.rank);
        let side = Vec3::new(k.rand(i as u32 + 40), 0.5, k.rand(i as u32 + 50)).normalize();
        icosa(&mut k.mesh, centre, radii, (0.22, seed), colour);
        spots.push(centre + side * r * 0.9);
    }
    spots
}

/// Adds two bare twigs forking from each of `tips`.
fn twigs(k: &mut Kit, tips: &[Vec3]) {
    for (i, &tip) in tips.iter().enumerate() {
        for s in 0..2 {
            let n = i as u32 * 2 + s + 70;
            let dir = Vec3::new(k.rand(n), 0.9, k.rand(n + 20)).normalize();
            k.stem(tip, tip + dir * 0.6, [0.03, 0.01]);
        }
    }
}

/// Adds the cones of needles of a conifer up its trunk from `base` along `up`, one more per canopy layer,
/// narrow or wide and drooping as its species, the lowest one shed when it is old.
fn conifer(k: &mut Kit, (base, up): (Vec3, Vec3), f: &Frame) {
    let (form, kind, colour) = (k.dials.form, k.dials.kind, k.leaves());
    let tiers = 2 + u32::from(form.layers.clamp(1, 4)) - u32::from(k.age == 0);
    let width = (0.6 + 0.65 * kind.spread) * f.crown;
    let flat = 1.0 - 0.35 * kind.droop;
    for t in u32::from(k.age == OLD)..tiers {
        let share = t as f32 / tiers as f32;
        let at = base + up * (1.3 + 3.0 * share);
        let size = Vec2::new(width * (1.0 - 0.7 * share), (1.4 - 0.5 * share) * flat);
        cone(&mut k.mesh, at, size, 7, colour);
    }
}
