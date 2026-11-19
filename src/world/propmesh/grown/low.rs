//! The low architectures: rosette, mat, vine, bulb and kelp.

use bevy::prelude::*;

use super::bits::{bead, spike};
use super::kit::Kit;
use crate::world::solids::{fan, icosa, prism};

/// Adds `count` spines in a ring round `centre`, each `length` long, pointing out at `rise` over the level,
/// `reach` from the centre and turned by `yaw`.
fn ring(k: &mut Kit, centre: Vec3, count: u32, (rise, reach, yaw): (f32, f32, f32), length: f32) {
    let spine = k.tone(k.dials.paint.stem);
    for b in 0..count {
        let a = yaw + b as f32 / count as f32 * std::f32::consts::TAU;
        let out = Vec3::new(a.cos(), rise, a.sin());
        spike(&mut k.mesh, centre + out * reach, out, length, spine);
    }
}

/// Builds a rosette: a ring of long blades round a centre, a second ring under wax, a swollen heart where it
/// stores water, a spine at each blade tip in drought, and a flower spike when it flowers.
pub fn rosette(k: &mut Kit) {
    let form = k.dials.form;
    let (colour, yaw) = (k.leaves(), k.rand(1) * 3.0);
    let width = 0.06 + 0.18 * form.leaf.min(1.4);
    let (outer, inner) = (
        [9.0, 0.6, 0.45, 0.12, width],
        [5.0, 0.35, 0.55, 0.08, width * 0.8],
    );
    fan(&mut k.mesh, Vec3::Y * 0.02, outer, yaw, colour);
    if form.wax {
        fan(&mut k.mesh, Vec3::Y * 0.04, inner, yaw + 0.3, colour);
    }
    if form.store {
        let heart = Vec3::new(0.25, 0.2, 0.25);
        icosa(&mut k.mesh, Vec3::Y * 0.15, heart, (0.15, 5), colour);
    }
    if form.spines {
        ring(k, Vec3::Y * 0.02, 9, (0.75, 0.55, yaw), 0.12);
    }
    if k.dials.paint.flowers.is_some() || k.dials.paint.fruit.is_some() {
        let head = Vec3::new(0.05 * k.rand(2), 1.3, 0.05 * k.rand(3));
        k.stem(Vec3::ZERO, head, [0.04, 0.03]);
        k.dress(&[head, head * 0.85, head * 0.7], 0.08);
    }
}

/// Builds a mat: two overlapping low films on the ground, and a cushion per extra layer.
pub fn mat(k: &mut Kit) {
    let form = k.dials.form;
    let colour = k.leaves();
    let deep = [colour[0] * 0.8, colour[1] * 0.8, colour[2] * 0.8, 1.0];
    let shift = Vec3::new(k.rand(1), 0.0, k.rand(2)) * 0.5;
    let wide = Vec2::new(0.8 + 0.15 * k.rand(3), 0.12);
    prism(&mut k.mesh, Vec3::Y * -0.08, wide, 5, colour);
    prism(
        &mut k.mesh,
        shift - Vec3::Y * 0.07,
        Vec2::new(0.55, 0.12),
        5,
        deep,
    );
    for c in 0..u32::from(form.layers.clamp(1, 3)) - 1 {
        let at = Vec3::new(k.rand(c + 5), 0.02, k.rand(c + 7)) * 0.5;
        icosa(&mut k.mesh, at, Vec3::new(0.3, 0.12, 0.3), (0.2, c), deep);
    }
    k.dress(&[shift + Vec3::Y * 0.08, Vec3::new(-0.3, 0.08, 0.2)], 0.06);
}

/// Builds a vine: a creeper that curls up from the ground in two stems, a leaf fan at each node, thorns
/// along it when armed, and flowers and fruit at its end.
pub fn vine(k: &mut Kit) {
    let form = k.dials.form;
    let colour = k.leaves();
    let nodes = [
        Vec3::new(0.0, -0.05, 0.0),
        Vec3::new(0.5, 0.45 + 0.15 * k.rand(1), 0.25 * k.rand(2)),
        Vec3::new(1.0 + 0.2 * k.rand(3), 0.35, -0.3 + 0.2 * k.rand(4)),
    ];
    for w in nodes.windows(2) {
        k.stem(w[0], w[1], [0.06, 0.04]);
    }
    if form.thorns {
        spike(&mut k.mesh, nodes[1], Vec3::Y, 0.12, colour);
        spike(&mut k.mesh, nodes[2], Vec3::X, 0.12, colour);
    }
    if k.dials.paint.leaves.is_some() {
        let size = form.leaf.min(1.4);
        let blade = [3.0, 0.3 * size + 0.1, 0.06, 0.0, 0.25 * size + 0.05];
        for (i, &at) in nodes.iter().enumerate() {
            fan(
                &mut k.mesh,
                at + Vec3::Y * 0.03,
                blade,
                i as f32 * 1.3,
                colour,
            );
        }
    }
    k.dress(&[nodes[2] + Vec3::Y * 0.1, nodes[1] + Vec3::Y * 0.1], 0.07);
}

/// Builds a bulb: a round body, swollen where it stores water, a tuft of blades on top, spines in drought,
/// and flowers and fruit over the tuft.
pub fn bulb(k: &mut Kit) {
    let form = k.dials.form;
    let colour = k.leaves();
    let swell = if form.store { 1.3 } else { 1.0 };
    let body = k.tone(k.dials.paint.stem);
    let round = Vec3::splat(0.3 * swell);
    icosa(&mut k.mesh, Vec3::Y * 0.22, round, (0.12, 9), body);
    let blade = [5.0, 0.12 + 0.3 * form.leaf.min(1.4), 0.35, 0.08, 0.12];
    let yaw = k.rand(1) * 3.0;
    fan(&mut k.mesh, Vec3::Y * (0.45 * swell), blade, yaw, colour);
    if form.spines {
        let yaw = k.rand(2);
        ring(k, Vec3::Y * 0.22, 3, (0.2, 0.27 * swell, yaw), 0.1);
    }
    let top = Vec3::Y * (0.8 * swell);
    k.dress(
        &[
            top,
            top + Vec3::new(0.15, -0.1, 0.0),
            top + Vec3::new(-0.1, -0.12, 0.1),
        ],
        0.06,
    );
}

/// Builds a kelp: a ribbon of three stems up from the floor, a pair of fronds at each joint and floats
/// below the top.
pub fn kelp(k: &mut Kit) {
    let form = k.dials.form;
    let colour = k.leaves();
    let nodes = [
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(0.2 * k.rand(1), 1.4, 0.1 * k.rand(2)),
        Vec3::new(-0.15 * k.rand(3), 2.8, 0.1 * k.rand(4)),
        Vec3::new(0.15 * k.rand(5), 4.0, 0.05),
    ];
    for w in nodes.windows(2) {
        k.stem(w[0], w[1], [0.07, 0.05]);
    }
    let frond = [2.0, 0.5 * form.leaf.min(1.4) + 0.2, 0.4, 0.0, 0.18];
    for (i, &at) in nodes[1..].iter().enumerate() {
        fan(&mut k.mesh, at, frond, i as f32 * 1.1, colour);
    }
    let float = k.tone(k.dials.paint.fruit.unwrap_or(k.dials.paint.stem));
    for &at in &nodes[2..] {
        bead(&mut k.mesh, at - Vec3::Y * 0.3, Vec3::splat(0.12), float);
    }
}
