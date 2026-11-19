//! Crystalline archetypes: the crystal hopper and the faceted roller. Both move as rigid bodies.

use std::f32::consts::TAU;

use bevy::math::Vec3;

use super::frame::aim;
use super::make::Make;
use super::piece::piece;
use super::{FeatureKind, PartKind, ShapeKind};

/// Number of crystalline archetypes.
pub const COUNT: u8 = 2;

/// Builds crystalline archetype `archetype`: 0 crystal hopper, 1 faceted roller.
pub fn build(m: &mut Make, archetype: u8) {
    match archetype % COUNT {
        0 => hopper(m),
        _ => roller(m),
    }
}

/// Builds a hopper: a crystal block crowned by two pairs of spikes, a prow and a pair of eyes.
fn hopper(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::new(0.45 * s * m.t.round, 0.45 * s, 0.7 * s);
    let block = piece(PartKind::Crystal, ShapeKind::Block, extent).at(Vec3::Y * 0.5 * extent.y);
    let body = m.kit.add(None, block);
    let length = 0.5 * s * m.vary(0.2);
    for z in [-0.25, 0.25] {
        let socket = Vec3::new(0.3 * extent.x, 0.4 * extent.y, z * extent.z);
        spike(m, body, (socket, Vec3::new(0.6, 1.0, z)), length);
    }
    let prow = piece(
        PartKind::Crystal,
        ShapeKind::Spike,
        Vec3::new(0.3, 0.6, 0.3) * s,
    )
    .at(Vec3::new(0.0, 0.1 * extent.y, -0.45 * extent.z))
    .turned(aim(Vec3::new(0.0, 0.3, -1.0)));
    m.kit.add(Some(body), prow.of(1, 1.0));
    let radius = (m.t.eye * extent.y * 0.5).min(0.15 * extent.x);
    let eye = piece(PartKind::Eye, ShapeKind::Ball, Vec3::splat(2.0 * radius)).at(Vec3::new(
        0.3 * extent.x,
        0.15 * extent.y,
        -0.5 * extent.z,
    ));
    m.kit.pair([body; 2], eye.of(2, 1.0));
}

/// Builds a roller: a faceted crystal ball studded with three pairs of spikes.
fn roller(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::splat(s * m.t.round);
    let ball = piece(PartKind::Crystal, ShapeKind::Ball, extent).at(Vec3::Y * 0.5 * extent.y);
    let body = m.kit.add(None, ball);
    let length = 0.35 * s * m.vary(0.2);
    for i in 0..3 {
        let angle = i as f32 * TAU / 3.0;
        let dir = Vec3::new(0.7, angle.cos(), angle.sin());
        spike(m, body, (dir * 0.4 * extent.x, dir), length);
    }
}

/// Adds a pair of crystal spikes of `length` on `body`, the right one at `socket` pointing along `dir`.
fn spike(m: &mut Make, body: u8, (socket, dir): (Vec3, Vec3), length: f32) {
    let width = 0.4 * length;
    let part = piece(
        PartKind::Crystal,
        ShapeKind::Spike,
        Vec3::new(width, length, width),
    )
    .at(socket)
    .turned(aim(dir));
    m.kit.pair([body; 2], part.of(1, 1.0));
    m.kit.feature(FeatureKind::Spike, length, 1);
}
