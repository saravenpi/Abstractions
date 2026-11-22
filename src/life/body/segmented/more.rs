//! Further segmented archetypes: the rearing mantid stalker and high-legged arachnid strider.

use bevy::math::Vec3;

use super::feelers;
use crate::life::body::frame::aim;
use crate::life::body::head::head;
use crate::life::body::leg::{LegSpec, legs, stance};
use crate::life::body::make::Make;
use crate::life::body::piece::piece;
use crate::life::body::{FeatureKind, Knee, PartKind, ShapeKind};

/// Adds the raised prothorax, strike claws, head and feelers of a mantid onto `torso`.
fn thorax(m: &mut Make, torso: u8, ext: Vec3, s: f32) {
    let reach = 0.52 * s * m.t.append;
    let neck = piece(
        PartKind::Neck,
        ShapeKind::Taper,
        Vec3::new(0.16 * s, reach, 0.16 * s),
    );
    let pose = (
        Vec3::new(0.0, 0.25 * ext.y, -0.42 * ext.z),
        aim(Vec3::new(0.0, 1.0, -0.35)),
    );
    let chest = m
        .kit
        .add(Some(torso), neck.at(pose.0).turned(pose.1).of(0, 0.85));
    let claw = piece(
        PartKind::Horn,
        ShapeKind::Spike,
        Vec3::new(0.08, 0.46, 0.1) * s,
    );
    let arm = (
        Vec3::new(0.11 * s, -0.75 * reach, 0.0),
        aim(Vec3::new(0.35, -0.85, -1.0)),
    );
    m.kit
        .pair([chest; 2], claw.at(arm.0).turned(arm.1).of(1, 1.0));
    m.kit.feature(FeatureKind::Spike, 0.46 * s, 1);
    let face = head(m, chest, Vec3::new(0.0, -reach, 0.0), 0.26 * s);
    feelers(m, face, 0.26 * s);
}

/// Builds a rearing mantid stalker: a low abdomen on four walking legs behind an upright prothorax
/// carrying a pair of forward-folded raptorial strike limbs and a crested head.
pub(super) fn mantid(m: &mut Make) {
    let (s, leg) = (m.t.size, 0.58 * m.t.size * m.t.reach * m.vary(0.1));
    let ext = Vec3::new(0.34 * m.t.round, 0.24 * m.t.hide, 0.68) * s;
    let body = piece(PartKind::Torso, ShapeKind::Ball, ext);
    let torso = m.kit.add(None, body.at(Vec3::Y * stance(leg, 0.52 * leg)));
    for (rank, z) in [(0u8, -0.15 * ext.z), (1, 0.28 * ext.z)] {
        let spec = LegSpec {
            hip: Vec3::new(0.36 * ext.x, 0.0, z),
            bones: [0.42 * leg, 0.58 * leg, 0.0],
            radius: 0.032 * s * m.t.thick,
            knee: Knee::Out,
            out: Vec3::X * 0.52 * leg,
            rank,
            shapes: [ShapeKind::Taper, ShapeKind::Spike],
        };
        legs(&mut m.kit, torso, &spec);
    }
    thorax(m, torso, ext, s);
}

/// Builds a high-legged arachnid strider: a compact domed carapace suspended on four pairs of tall,
/// wide-splayed jointed legs that tower over the ground.
pub(super) fn arachnid(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::new(0.42 * s * m.t.round, 0.34 * s * m.t.hide, 0.52 * s);
    let leg = 0.85 * s * m.t.reach * m.vary(0.1);
    let splay = 0.75 * leg;
    let hip = Vec3::new(0.36 * extent.x, 0.0, 0.0);
    let dome =
        piece(PartKind::Torso, ShapeKind::Ball, extent).at(Vec3::Y * (stance(leg, splay) - hip.y));
    let torso = m.kit.add(None, dome);
    for (rank, z) in [-0.36, -0.12, 0.12, 0.36].into_iter().enumerate() {
        let spec = LegSpec {
            hip: hip + Vec3::Z * z * extent.z,
            bones: [0.46 * leg, 0.54 * leg, 0.0],
            radius: 0.032 * s * m.t.thick,
            knee: Knee::Out,
            out: Vec3::new(splay, 0.0, 0.35 * z * splay),
            rank: rank as u8,
            shapes: [ShapeKind::Spike; 2],
        };
        legs(&mut m.kit, torso, &spec);
    }
    let index = head(
        m,
        torso,
        Vec3::new(0.0, -0.05 * extent.y, -0.46 * extent.z),
        0.24 * s,
    );
    feelers(m, index, 0.24 * s);
}
