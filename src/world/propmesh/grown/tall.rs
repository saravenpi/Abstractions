//! The tall architectures: trunk and branches, stalk and cap, and the crystal lattice. Each starts half a
//! metre under the ground so that a slope never shows its underside.

use bevy::prelude::*;

use super::bits::bead;
use super::kit::Kit;
use crate::world::solids::{cone, icosa};

/// Returns the unit direction at `angle` radians round the vertical, lifted by `rise`.
fn round(angle: f32, rise: f32) -> Vec3 {
    Vec3::new(angle.cos(), rise, angle.sin())
}

/// Adds the canopy of a broad leaved tree over `top`: one tier per layer, two clumps in each but the
/// highest, and returns the spots its flowers and fruit hang on.
fn crown(k: &mut Kit, top: Vec3) -> Vec<Vec3> {
    let form = k.dials.form;
    let (tiers, colour) = (u32::from(form.layers.clamp(1, 4)), k.leaves());
    let size = 0.75 + 0.25 * form.leaf.min(1.4);
    let mut spots = Vec::new();
    for t in 0..tiers {
        let y = top.y + 0.4 + 0.55 * t as f32;
        let r = (1.5 - 0.25 * t as f32) * size;
        let clumps = if t + 1 == tiers { 1 } else { 2 };
        for c in 0..clumps {
            let side = round(k.rand(t * 4 + c) * 3.2 + c as f32 * 3.1, 0.0) * r * 0.45 * c as f32;
            let at = Vec3::new(top.x, y, top.z) + side;
            let radii = Vec3::new(r, r * 0.7, r);
            icosa(&mut k.mesh, at, radii, (0.2, t * 7 + c), colour);
            spots.push(at + round(k.rand(t + 40) * 3.0, 0.4).normalize() * r * 0.9);
        }
    }
    spots.truncate(5);
    spots
}

/// Builds a tree: a buried trunk swollen where it stores water, two branches, then cones of needles in the
/// cold, a layered crown of leaves elsewhere, or bare twigs while it is dormant.
pub fn trunk(k: &mut Kit) {
    let form = k.dials.form;
    let girth = if form.store { 0.55 } else { 0.28 };
    let base = Vec3::new(0.0, -0.5, 0.0);
    let top = Vec3::new(0.15 * k.rand(1), 2.2, 0.15 * k.rand(2));
    k.stem(base, top, [girth, girth * 0.6]);
    k.armour(Vec3::ZERO, top, girth * 0.7);
    if form.needles {
        let colour = k.leaves();
        for t in 0..u32::from(form.layers.clamp(1, 4)) + 1 {
            let at = Vec3::new(top.x, 0.9 + 0.75 * t as f32, top.z);
            cone(
                &mut k.mesh,
                at,
                Vec2::new(1.6 - 0.3 * t as f32, 1.3),
                7,
                colour,
            );
        }
        return;
    }
    let bare = k.dials.paint.leaves.is_none();
    let arms = if bare { 4 } else { 2 };
    for a in 0..arms {
        let tip = top + round(a as f32 * 1.9 + k.rand(a + 3), 0.9) * (0.9 + 0.2 * k.rand(a + 9));
        k.stem(top * 0.8, tip, [0.12 * girth / 0.28, 0.05]);
    }
    if !bare {
        let spots = crown(k, top);
        k.dress(&spots, 0.12);
    }
}

/// Builds a stalk and cap: a buried stalk under one cap per layer, flowers on the rims and fruit beneath.
pub fn stalk(k: &mut Kit) {
    let form = k.dials.form;
    let girth = if form.store { 0.42 } else { 0.2 };
    let top = Vec3::new(0.2 * k.rand(1), 3.0, 0.2 * k.rand(2));
    k.stem(Vec3::new(0.0, -0.5, 0.0), top, [girth, girth * 0.75]);
    k.armour(Vec3::ZERO, top, girth * 0.8);
    if form.store {
        let body = k.tone(k.dials.paint.stem);
        icosa(
            &mut k.mesh,
            top * 0.45,
            Vec3::splat(girth * 1.8),
            (0.15, 3),
            body,
        );
    }
    let colour = k.leaves();
    let mut spots = Vec::new();
    for t in 0..u32::from(form.layers.clamp(1, 4)) {
        let r = (1.3 - 0.25 * t as f32) * (0.8 + 0.2 * form.leaf.min(1.4));
        let at = top - Vec3::Y * 0.75 * t as f32;
        cone(&mut k.mesh, at, Vec2::new(r, 0.45), 7, colour);
        spots.push(at + round(k.rand(t + 20) * 3.0, 0.0) * r * 0.95);
    }
    k.dress(&spots, 0.12);
}

/// Builds a crystal lattice: buried shards leaning out of one root, one more per layer, each tipped with a
/// facet of the pigment.
pub fn lattice(k: &mut Kit) {
    let form = k.dials.form;
    let tip = k.leaves();
    for s in 0..3 + u32::from(form.layers.clamp(1, 4)) {
        let lean = if s == 0 { 0.0 } else { 0.5 + 0.25 * k.rand(s) };
        let dir = round(s as f32 * 2.3 + k.rand(s + 10), 1.0 / lean.max(0.2)).normalize();
        let length = if s == 0 {
            2.9
        } else {
            1.4 + 0.5 * k.rand(s + 20)
        };
        let from = Vec3::new(0.0, -0.4, 0.0) + dir * 0.1;
        let to = from + dir * length;
        let girth = if s == 0 { 0.28 } else { 0.18 };
        k.stem(from, to, [girth, girth * 0.3]);
        bead(
            &mut k.mesh,
            to,
            Vec3::new(girth * 0.6, girth * 1.2, girth * 0.6),
            tip,
        );
    }
}
