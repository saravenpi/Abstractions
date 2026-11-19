//! Radial archetypes: the floating jelly bell with its tentacles and the star walker on its arms.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};

use super::frame::{aim, mirror};
use super::leg::{LegSpec, legs, stance};
use super::make::Make;
use super::piece::piece;
use super::{Knee, Part, PartKind, ShapeKind};
use crate::life::Habitat;

/// Number of radial archetypes.
pub const COUNT: u8 = 2;

/// Largest bend per joint of a tentacle, in radians.
const BEND: f32 = 0.8;

/// Builds radial archetype `archetype`: 0 floating jelly, 1 star walker.
pub fn build(m: &mut Make, archetype: u8) {
    match archetype % COUNT {
        0 => jelly(m),
        _ => star(m),
    }
}

/// Returns the directions in the ground plane of the arms on the right side of a body with `count` arms,
/// from front to back. Arms sit half a step off the axis, so each one has a twin on the left.
fn spokes(count: usize) -> Vec<Vec3> {
    (0..count / 2)
        .map(|i| (i as f32 + 0.5) * TAU / count as f32)
        .map(|angle| Vec3::new(angle.sin(), 0.0, -angle.cos()))
        .collect()
}

/// Returns the links of one tentacle on the right side hanging from `socket` toward `dir`.
fn tentacle(socket: Vec3, dir: Vec3, (count, link, width): (usize, f32, f32)) -> Vec<Part> {
    (0..count)
        .map(|i| {
            let (at, rest) = match i {
                0 => (socket, aim(dir)),
                _ => (Vec3::new(0.0, -link, 0.0), Quat::IDENTITY),
            };
            let thin = width * (1.0 - 0.25 * i as f32);
            let size = Vec3::new(thin, link, thin);
            let part = piece(PartKind::Tentacle, ShapeKind::Taper, size)
                .at(at)
                .turned(rest)
                .of(0, 0.7);
            Part { side: 1, ..part }
        })
        .collect()
}

/// Builds a jelly: a bell floating over the ground, or high in the air for a gas bag, trailing tentacles.
fn jelly(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::new(s * m.t.round, 0.6 * s, s * m.t.round);
    let count = if m.chance(0.5) { 4 } else { 6 };
    let links = if count == 4 { 3 } else { 2 };
    let length = 1.2 * s * m.t.append * m.vary(0.15);
    let lift = if m.habitat == Habitat::Air {
        2.0 * s
    } else {
        0.0
    };
    let bell =
        piece(PartKind::Bell, ShapeKind::Ball, extent).at(Vec3::Y * (length + 0.3 * s + lift));
    let bell = m.kit.add(None, bell.of(0, 1.0));
    for dir in spokes(count) {
        let socket = Vec3::new(
            0.4 * extent.x * dir.x,
            -0.3 * extent.y,
            0.4 * extent.z * dir.z,
        );
        let links = tentacle(
            socket,
            dir * 0.3 + Vec3::NEG_Y,
            (links, length / links as f32, 0.07 * s),
        );
        let left: Vec<Part> = links.iter().map(|&p| mirror(p)).collect();
        let left = m.kit.chain(Some(bell), &left, BEND);
        let right = m.kit.chain(Some(bell), &links, BEND);
        m.kit.body.tentacles.extend([left, right]);
    }
}

/// Builds a star walker: a flat disc on four or six splayed arms.
fn star(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::new(0.5 * s, 0.2 * s, 0.5 * s) * m.t.round;
    let count = if m.chance(0.5) { 4 } else { 6 };
    let leg = 0.45 * s * m.t.reach * m.vary(0.1);
    let splay = 0.5 * leg;
    let drop = -0.2 * extent.y;
    let disc =
        piece(PartKind::Torso, ShapeKind::Ball, extent).at(Vec3::Y * (stance(leg, splay) - drop));
    let disc = m.kit.add(None, disc);
    for (rank, dir) in spokes(count).into_iter().enumerate() {
        let spec = LegSpec {
            hip: dir * 0.42 * extent.x + Vec3::Y * drop,
            bones: [0.45 * leg, 0.55 * leg, 0.0],
            radius: 0.04 * s * m.t.thick,
            knee: Knee::Out,
            out: dir * splay,
            rank: rank as u8,
            shapes: [ShapeKind::Taper, ShapeKind::Spike],
        };
        legs(&mut m.kit, disc, &spec);
    }
}
