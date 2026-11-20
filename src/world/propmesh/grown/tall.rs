//! The tall architectures besides the tree: stalk and cap, and the crystal lattice. Each starts half a metre
//! under the ground so that a slope never shows its underside.

use bevy::prelude::*;

use super::bits::{bead, spike};
use super::kit::{Kit, OLD};
use crate::world::solids::{cone, icosa};

/// Returns the unit direction at `angle` radians round the vertical, lifted by `rise`.
fn round(angle: f32, rise: f32) -> Vec3 {
    Vec3::new(angle.cos(), rise, angle.sin())
}

/// Returns how wide and how tall the caps of `k` are against a grown one: a closed bell when young, wide and
/// flat when old.
fn bell(k: &Kit) -> (f32, f32) {
    match k.age {
        0 => (0.55, 1.6),
        OLD => (1.2, 0.8),
        _ => (1.0, 1.0),
    }
}

/// Adds `tiers` caps over the stalk top `top`, as wide as the species spreads and hanging as far as it
/// droops, and pushes onto `spots` where a flower sits on each rim.
fn caps(k: &mut Kit, top: Vec3, tiers: u32, spots: &mut Vec<Vec3>) {
    let (form, kind, colour) = (k.dials.form, k.dials.kind, k.leaves());
    let (wide, tall) = bell(k);
    for t in 0..tiers {
        let r = (1.3 - 0.25 * t as f32)
            * (0.8 + 0.2 * form.leaf.min(1.4))
            * (0.6 + 0.45 * kind.spread)
            * wide;
        let h = (0.45 + 0.6 * kind.droop) * tall;
        let at = top - Vec3::Y * (0.75 * t as f32 + 0.5 * h * kind.droop);
        cone(&mut k.mesh, at, Vec2::new(r, h), 7, colour);
        spots.push(at + round(k.rand(t + 20) * 3.0, 0.0) * r * 0.95);
    }
}

/// Adds stalk number `s` of a stalk and cap with a foot `girth` metres thick: the leading one upright and
/// full height, the others shorter, thinner and splayed beside it, and returns its top.
fn shoot(k: &mut Kit, s: u32, girth: f32) -> Vec3 {
    let first = usize::from(s == 0);
    let side = round(s as f32 * 2.6 + k.rand(s + 44), 0.0) * 0.9 * girth * (1 - first) as f32;
    let up = (k.axis() + side * 0.6).normalize();
    let tall = [1.4 + 0.8 * k.rand(s + 40).abs(), 3.0][first];
    let g = girth * [0.6, 1.0][first];
    let foot = Vec3::new(0.0, -0.5, 0.0) + side;
    let top = foot + up * (tall + 0.5);
    k.stem(foot, top, [g, g * 0.75]);
    top
}

/// Builds a stalk and cap: a buried leading stalk under one cap per layer, up to two shorter stalks splayed
/// beside it with a cap each, a swollen store where it keeps water, flowers on the rims and fruit beneath.
pub fn stalk(k: &mut Kit) {
    let (form, kind) = (k.dials.form, k.dials.kind);
    let girth = [0.2, 0.42][usize::from(form.store)] * k.grow;
    let count = 1 + u32::from(k.age > 0) * u32::from(kind.branches.saturating_sub(2)).min(2);
    let mut spots = Vec::new();
    let lead = shoot(k, 0, girth);
    caps(k, lead, u32::from(form.layers.clamp(1, 4)), &mut spots);
    for s in 1..count {
        let top = shoot(k, s, girth);
        caps(k, top, 1, &mut spots);
    }
    k.armour(Vec3::ZERO, lead, girth * 0.8);
    if form.store {
        let body = k.tone(k.dials.paint.stem);
        icosa(
            &mut k.mesh,
            lead * 0.45,
            Vec3::splat(girth * 1.8),
            (0.15, 3),
            body,
        );
    }
    k.dress(&spots, 0.12);
}

/// Returns how many shards the lattice of `k` grows: few when young, the most when old.
fn shards(k: &Kit) -> u32 {
    let layers = u32::from(k.dials.form.layers.clamp(1, 4));
    match k.age {
        0 => 2,
        OLD => 4 + layers.min(3),
        _ => (2 + u32::from(k.dials.kind.branches.min(4)) + layers / 2).min(7),
    }
}

/// Adds shard number `s` of a lattice leaning from `up` by up to `splay` radians, tipped with a facet of
/// `tip`, and a needle of crystal along one of the first three of a many leaved species.
fn shard(k: &mut Kit, s: u32, (up, splay): (Vec3, f32), tip: [f32; 4]) {
    let (kind, first) = (k.dials.kind, usize::from(s == 0));
    let lean = (1 - first) as f32 * splay * (0.7 + 0.5 * k.rand(s).abs());
    let a = s as f32 * 2.4 + k.rand(s + 10);
    let dir = (up * lean.cos() + round(a, 0.0) * lean.sin()).normalize();
    let length = [1.2 + 0.7 * k.rand(s + 20).abs(), 2.9][first] * k.grow;
    let girth = [0.18, 0.28][first] * (0.75 + 0.3 * kind.spread);
    let from = Vec3::new(0.0, -0.5, 0.0) + dir * 0.1;
    let to = from + dir * length;
    k.stem(from, to, [girth, girth * 0.3]);
    bead(&mut k.mesh, to, Vec3::new(0.6, 1.2, 0.6) * girth, tip);
    if s < 3 && kind.leaves > 6 {
        let needle = dir + round(a + 1.6, 0.2);
        spike(&mut k.mesh, from.lerp(to, 0.6), needle, 0.35 * length, tip);
    }
}

/// Builds a crystal lattice: buried shards leaning out of one root as wide as the species splays them.
pub fn lattice(k: &mut Kit) {
    let (tip, splay) = (k.leaves(), 0.25 + 0.45 * k.dials.kind.spread);
    let up = k.axis();
    for s in 0..shards(k) {
        shard(k, s, (up, splay), tip);
    }
}
