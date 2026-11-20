//! The bulb and the kelp: a swollen body under a tuft of blades, and ribbons of stipes under the liquid.

use bevy::prelude::*;

use super::bits::bead;
use super::kit::Kit;
use super::low::{level, ring};
use crate::world::solids::{fan, icosa};

/// Adds two small offsets beside a bulb of `round` radii in colour `body`, when its species clumps and it
/// is past its youth.
fn offsets(k: &mut Kit, round: Vec3, body: [f32; 4]) {
    if k.dials.kind.branches < 4 || k.age == 0 {
        return;
    }
    for s in 0..2 {
        let at = (level(s as f32 * 3.1 + k.rand(s + 30)) + Vec3::Y * 0.1) * round.x * 1.3;
        bead(&mut k.mesh, at, Vec3::splat(0.4 * round.y), body);
    }
}

/// Returns the fan of blades over a bulb: as many as its species has leaves, arched as it droops when there
/// are few of them, flat when there are many.
fn tuft(k: &Kit) -> [f32; 5] {
    let (form, kind) = (k.dials.form, k.dials.kind);
    let count = f32::from(kind.leaves);
    let arch = f32::from(u8::from(count <= 6.0)) * (0.08 + 0.2 * kind.droop);
    let reach = 0.12 + 0.3 * form.leaf.min(1.4);
    [count, reach, 0.35 - 0.2 * kind.droop, arch, 0.12]
}

/// Builds a bulb: a round body, squat or tall as its species and swollen where it stores water, offsets
/// beside it when it clumps, a tuft of blades on top, spines in drought, and flowers and fruit over the tuft.
pub fn bulb(k: &mut Kit) {
    let (form, kind, colour) = (k.dials.form, k.dials.kind, k.leaves());
    let swell = [1.0, 1.3][usize::from(form.store)] * k.grow;
    let (squat, body) = (kind.spread.sqrt(), k.tone(k.dials.paint.stem));
    let round = Vec3::new(0.3 * squat, 0.3 / squat, 0.3 * squat) * swell;
    let heart = Vec3::Y * round.y * 0.75;
    icosa(&mut k.mesh, heart, round, (0.12, 9), body);
    offsets(k, round, body);
    let (crown, blade, yaw) = (Vec3::Y * round.y * 1.5, tuft(k), k.rand(1) * 3.0);
    fan(&mut k.mesh, crown, blade, yaw, colour);
    if form.spines {
        let yaw = k.rand(2);
        ring(k, heart, 3, (0.2, round.x * 0.9, yaw), 0.1);
    }
    let top = crown + Vec3::Y * 0.35;
    k.dress(&[top, top + Vec3::new(0.15, -0.1, 0.0)], 0.06);
}

/// Adds stipe number `s` of a kelp: a ribbon of three segments from the floor, the first full height and the
/// others shorter, with fronds of `frond` at each joint and a float on the first.
fn stipe(k: &mut Kit, s: u32, frond: [f32; 5]) {
    let colour = k.leaves();
    let tall = [0.6 + 0.25 * k.rand(s + 30).abs(), 1.0][usize::from(s == 0)];
    let a = s as f32 * 2.3;
    let foot = level(a) * 0.15 * s as f32 - Vec3::Y * 0.1;
    let sway = |i: u32| Vec3::new(0.2 * k.rand(i), 0.0, 0.15 * k.rand(i + 1));
    let nodes = [1u32, 2, 3].map(|n| foot + sway(s * 8 + n) + Vec3::Y * 1.35 * n as f32 * tall);
    let radii = [[0.07, 0.06], [0.06, 0.05], [0.05, 0.04]];
    for (i, (&at, r)) in nodes.iter().zip(radii).enumerate() {
        k.stem(if i == 0 { foot } else { nodes[i - 1] }, at, r);
        fan(&mut k.mesh, at, frond, i as f32 * 1.1 + a, colour);
    }
    if s == 0 {
        let float = k.tone(k.dials.paint.fruit.unwrap_or(k.dials.paint.stem));
        bead(
            &mut k.mesh,
            nodes[1] - Vec3::Y * 0.3,
            Vec3::splat(0.12),
            float,
        );
    }
}

/// Builds a kelp: one stipe when young and up to three of unequal height later, with fronds that trail as
/// far as the species droops.
pub fn kelp(k: &mut Kit) {
    let (form, kind) = (k.dials.form, k.dials.kind);
    let stipes = [1, u32::from(kind.branches.clamp(1, 3))][usize::from(k.age > 0)];
    let frond = [
        2.0,
        0.5 * form.leaf.min(1.4) + 0.2,
        0.4 - 0.6 * kind.droop,
        0.0,
        0.18,
    ];
    for s in 0..stipes {
        stipe(k, s, frond);
    }
}
