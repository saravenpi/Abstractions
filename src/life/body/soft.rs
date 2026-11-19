//! Soft archetypes. Amorphous: the slime blob and the stacked blob. Colonial: the members of a swarm,
//! a drifting mote and a bead with feelers. The swarm is many of them.

use bevy::math::Vec3;

use super::frame::aim;
use super::make::Make;
use super::piece::piece;
use super::{PartKind, ShapeKind};

/// Number of amorphous archetypes.
pub const AMORPHOUS: u8 = 2;

/// Number of colonial archetypes.
pub const COLONIAL: u8 = 2;

/// Longest swarm member in metres.
const MEMBER: f32 = 0.12;

/// Builds amorphous archetype `archetype`: 0 slime blob, 1 stacked blob.
pub fn amorphous(m: &mut Make, archetype: u8) {
    let s = m.t.size;
    let extent = Vec3::new(s * m.t.round, 0.55 * s, 1.1 * s * m.t.round);
    let base = piece(PartKind::Blob, ShapeKind::Ball, extent).at(Vec3::Y * 0.5 * extent.y);
    let mut top = (m.kit.add(None, base), extent);
    if archetype % AMORPHOUS == 1 {
        let count = 1 + m.rng.below(2);
        for _ in 0..count {
            let size = top.1 * 0.7;
            let socket = Vec3::Y * (0.3 * top.1.y + 0.35 * size.y);
            let next = piece(PartKind::Blob, ShapeKind::Ball, size).at(socket);
            top = (m.kit.add(Some(top.0), next), size);
        }
    }
    eyes(m, top.0, top.1);
}

/// Adds a pair of eyes on the top front of a soft body `body` of extent `extent`.
fn eyes(m: &mut Make, body: u8, extent: Vec3) {
    let radius = (m.t.eye * 0.5 * extent.y).min(0.2 * extent.x);
    let socket = Vec3::new(0.2 * extent.x, 0.4 * extent.y, -0.3 * extent.z);
    let eye = piece(PartKind::Eye, ShapeKind::Ball, Vec3::splat(2.0 * radius)).at(socket);
    m.kit.pair([body; 2], eye.of(2, 1.0));
}

/// Builds colonial archetype `archetype`: 0 drifting mote with fins, 1 bead with a tail and feelers.
pub fn colonial(m: &mut Make, archetype: u8) {
    let s = m.t.size.min(MEMBER);
    let extent = Vec3::new(0.6 * s, 0.5 * s, s);
    let body = piece(PartKind::Torso, ShapeKind::Ball, extent).at(Vec3::Y * 0.5 * extent.y);
    let body = m.kit.add(None, body);
    match archetype % COLONIAL {
        0 => mote(m, body, s),
        _ => bead(m, body, s),
    }
}

/// Adds the pair of fins a mote rows with.
fn mote(m: &mut Make, body: u8, s: f32) {
    let fin = piece(
        PartKind::Fin,
        ShapeKind::Fan,
        Vec3::new(0.02 * s, 0.6 * s, 0.5 * s),
    )
    .at(Vec3::new(0.25 * s, 0.1 * s, 0.0))
    .turned(aim(Vec3::new(1.0, 0.3, 0.0)));
    m.kit.pair([body; 2], fin.of(1, 1.0));
}

/// Adds the tail segment and the feelers of a bead.
fn bead(m: &mut Make, body: u8, s: f32) {
    let tail = piece(
        PartKind::Segment,
        ShapeKind::Taper,
        Vec3::new(0.4 * s, 0.6 * s, 0.35 * s),
    )
    .at(Vec3::new(0.0, 0.0, 0.4 * s))
    .turned(aim(Vec3::new(0.0, -0.2, 1.0)));
    m.kit.add(Some(body), tail.of(0, 0.5));
    let feeler = piece(
        PartKind::Antenna,
        ShapeKind::Spike,
        Vec3::new(0.05 * s, 0.7 * s, 0.05 * s),
    )
    .at(Vec3::new(0.12 * s, 0.15 * s, -0.4 * s))
    .turned(aim(Vec3::new(0.4, 0.7, -1.0)));
    m.kit.pair([body; 2], feeler.of(1, 1.0));
}
