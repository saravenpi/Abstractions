//! Giant and predatory skeletal archetypes: the armoured siege charger, the apex theropod predator,
//! the skeletal hexapod stalker and the sail-backed behemoth.

use bevy::math::Vec3;

use super::super::frame::aim;
use super::super::head::{head, horns};
use super::super::leg::{LegSpec, legs, seat, stance};
use super::super::make::Make;
use super::super::piece::piece;
use super::super::trunk::{club, neck, plates, sail, tail};
use super::super::{FeatureKind, Knee, PartKind, ShapeKind};
use super::{LEG, biped};

/// Returns whether the body under construction already lists a feature of `kind`.
fn has(m: &Make, kind: FeatureKind) -> bool {
    m.kit.body.features.iter().any(|f| f.kind == kind)
}

/// Adds a heavy quadruped torso of `extent` on four pillar legs of length `leg` and thigh radius
/// `radius`, with front knees back and hind knees forward. Returns the torso index.
fn four(m: &mut Make, extent: Vec3, leg: f32, radius: f32) -> u8 {
    let body = piece(PartKind::Torso, ShapeKind::Ball, extent);
    let hip = seat(
        &body,
        Vec3::new(0.32 * extent.x, -0.3 * extent.y, 0.34 * extent.z),
        radius,
    );
    let torso = m
        .kit
        .add(None, body.at(Vec3::Y * (stance(leg, 0.0) - hip.y)));
    let mut spec = LegSpec {
        hip,
        bones: [0.48 * leg, 0.52 * leg, 0.0],
        radius,
        knee: Knee::Back,
        out: Vec3::ZERO,
        rank: 0,
        shapes: [ShapeKind::Taper; 2],
    };
    for (rank, z, knee) in [(0, -1.0, Knee::Back), (1, 1.0, Knee::Forward)] {
        (spec.hip.z, spec.knee, spec.rank) = (hip.z * z, knee, rank);
        legs(&mut m.kit, torso, &spec);
    }
    torso
}

/// Builds an armoured siege charger: a wide barrel body on thick pillar legs, horned skull, dorsal
/// armour plates and a heavy clubbed tail.
pub(super) fn charger(m: &mut Make) {
    let (s, girth) = (m.t.size, m.t.round * m.t.hide);
    let extent = Vec3::new(0.46 * s * girth, 0.4 * s * girth, 0.76 * s * m.vary(0.1));
    let leg = 0.78 * LEG * s * m.t.reach * m.vary(0.1);
    let torso = four(m, extent, leg, 0.068 * s * m.t.thick);
    let socket = Vec3::new(0.0, 0.12 * extent.y, -0.44 * extent.z);
    let (tip, link) = neck(
        m,
        torso,
        (socket, Vec3::new(0.0, 0.35, -1.0)),
        (1, 0.24 * s * m.t.append),
    );
    let skull = head(m, tip, Vec3::new(0.0, -link, 0.0), 0.34 * s);
    if !has(m, FeatureKind::Horn) {
        horns(m, skull, Vec3::splat(0.32 * s), 1.3);
    }
    plates(m, torso, extent);
    let tail_at = Vec3::new(0.0, 0.1 * extent.y, 0.46 * extent.z);
    tail(m, torso, tail_at, (2, 0.55 * s * m.t.append));
    let end = m
        .kit
        .body
        .tail
        .as_ref()
        .and_then(|t| t.parts.last().copied().zip(t.lengths.last().copied()));
    if let Some((tip, len)) = end
        && !has(m, FeatureKind::Spike)
    {
        club(m, tip, len);
    }
}

/// Builds an apex theropod predator: a heavy-jawed biped on muscular three-bone legs with short
/// clawed forelimbs and a long balancing tail.
pub(super) fn theropod(m: &mut Make) {
    let s = m.t.size;
    let leg = 0.88 * s * m.t.reach * m.vary(0.1);
    let extent = Vec3::new(0.36, 0.4 * m.t.hide, 0.72 / m.t.round) * s * m.t.round;
    let torso = biped(m, extent, (leg, [0.35, 0.39, 0.26], 0.065 * s * m.t.thick));
    let socket = Vec3::new(0.0, 0.28 * extent.y, -0.42 * extent.z);
    let (tip, link) = neck(
        m,
        torso,
        (socket, Vec3::new(0.0, 0.7, -1.0)),
        (2, 0.42 * s * m.t.append),
    );
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.36 * s);
    if m.kit.fits(2) {
        let at = Vec3::new(0.34 * extent.x, -0.15 * extent.y, -0.3 * extent.z);
        let claw = piece(
            PartKind::Horn,
            ShapeKind::Spike,
            Vec3::new(0.07, 0.28, 0.07) * s,
        )
        .at(at)
        .turned(aim(Vec3::new(0.15, -0.8, -0.8)));
        m.kit.pair([torso; 2], claw.of(1, 1.0));
    }
    let tail_at = Vec3::new(0.0, 0.1 * extent.y, 0.46 * extent.z);
    tail(m, torso, tail_at, (3, 0.9 * s * m.t.append));
    if m.chance(0.35) {
        sail(m, torso, extent, 0.28 * s * m.t.append);
    }
}

/// Builds a skeletal hexapod stalker: a sleek body on six wide-striding legs, a sinuous neck with
/// a horned head and a long whip tail.
pub(super) fn basilisk(m: &mut Make) {
    let (s, girth) = (m.t.size, m.t.round * m.t.hide);
    let extent = Vec3::new(0.34 * s * girth, 0.28 * s * girth, 0.86 * s * m.vary(0.1));
    let leg = 0.72 * s * m.t.reach * m.vary(0.1);
    let splay = 0.32 * leg;
    let body = piece(PartKind::Torso, ShapeKind::Ball, extent).at(Vec3::Y * stance(leg, splay));
    let torso = m.kit.add(None, body);
    let mut spec = LegSpec {
        hip: Vec3::new(0.35 * extent.x, 0.0, 0.0),
        bones: [0.46 * leg, 0.54 * leg, 0.0],
        radius: 0.042 * s * m.t.thick,
        knee: Knee::Out,
        out: Vec3::X * splay,
        rank: 0,
        shapes: [ShapeKind::Taper; 2],
    };
    for (rank, z) in [-0.34, 0.0, 0.34].into_iter().enumerate() {
        (spec.hip.z, spec.rank) = (z * extent.z, rank as u8);
        legs(&mut m.kit, torso, &spec);
    }
    let socket = Vec3::new(0.0, 0.22 * extent.y, -0.44 * extent.z);
    let at = (socket, Vec3::new(0.0, 0.75, -0.7));
    let (tip, link) = neck(m, torso, at, (2, 0.65 * s * m.t.append));
    let skull = head(m, tip, Vec3::new(0.0, -link, 0.0), 0.28 * s);
    if !has(m, FeatureKind::Horn) {
        horns(m, skull, Vec3::splat(0.26 * s), 1.1);
    }
    let tail_at = Vec3::new(0.0, 0.1 * extent.y, 0.46 * extent.z);
    tail(m, torso, tail_at, (2, 0.8 * s * m.t.append));
}

/// Builds a sail-backed behemoth: a massive quadruped with a tall dorsal spine sail, sturdy neck
/// and long sweeping tail.
pub(super) fn behemoth(m: &mut Make) {
    let (s, girth) = (m.t.size, m.t.round * m.t.hide);
    let extent = Vec3::new(0.42 * s * girth, 0.44 * s * girth, 0.82 * s * m.vary(0.1));
    let leg = 1.15 * LEG * s * m.t.reach * m.vary(0.1);
    let torso = four(m, extent, leg, 0.062 * s * m.t.thick);
    sail(m, torso, extent, 0.48 * s * m.t.append);
    if m.chance(0.35) {
        plates(m, torso, extent);
    }
    let at = (
        Vec3::new(0.0, 0.25 * extent.y, -0.44 * extent.z),
        Vec3::new(0.0, 0.75, -1.0),
    );
    let (tip, link) = neck(m, torso, at, (2, 0.55 * s * m.t.append));
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.28 * s);
    tail(
        m,
        torso,
        Vec3::new(0.0, 0.12 * extent.y, 0.46 * extent.z),
        (3, 0.8 * s * m.t.append),
    );
}
