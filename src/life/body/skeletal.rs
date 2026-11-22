//! Skeletal archetypes: the quadruped grazer, the towering titan browser, the agile biped runner,
//! the winged flyer (or terror-bird strider), the armoured siege charger, the apex theropod predator,
//! the skeletal hexapod stalker and the sail-backed behemoth.

mod giants;

use bevy::math::Vec3;

use super::frame::aim;
use super::head::head;
use super::leg::{LegSpec, legs, seat, stance};
use super::make::Make;
use super::piece::piece;
use super::trunk::{armour, flat, neck, tail};
use super::{Knee, PartKind, ShapeKind, Wings};
use crate::life::Guild;

/// Number of skeletal archetypes.
pub const COUNT: u8 = 8;

/// Leg length of a quadruped over its body length scale.
pub(super) const LEG: f32 = 0.75;

/// Builds skeletal archetype `archetype`: 0 quadruped grazer, 1 towering titan browser, 2 agile biped
/// runner, 3 winged flyer (or flightless terror-bird strider), 4 armoured siege charger, 5 apex
/// theropod predator, 6 skeletal hexapod stalker, 7 sail-backed behemoth.
pub fn build(m: &mut Make, archetype: u8) {
    match archetype % COUNT {
        0 => quadruped(m, false),
        1 => quadruped(m, true),
        2 => runner(m),
        3 if m.t.wing.is_some() => flyer(m),
        3 => strider(m),
        4 => giants::charger(m),
        5 => giants::theropod(m),
        6 => giants::basilisk(m),
        _ => giants::behemoth(m),
    }
}

/// Builds a four-legged body, with a towering 3-to-4 segment neck and pillar legs for a titan browser.
fn quadruped(m: &mut Make, browser: bool) {
    let s = m.t.size;
    let (torso, extent) = stand(m, browser);
    let long = browser || m.guild == Guild::Browser;
    let (count, reach, dir) = match long {
        true => (4, 1.3 * s, Vec3::new(0.0, 1.0, -0.35)),
        false => (1, 0.3 * s, Vec3::new(0.0, 0.55, -1.0)),
    };
    let socket = Vec3::new(0.0, 0.28 * extent.y, -0.42 * extent.z);
    let (tip, link) = neck(m, torso, (socket, dir), (count, reach * m.t.append));
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.26 * s);
    let tail_len = if browser { 0.65 } else { 0.45 } * s * m.t.append;
    tail(
        m,
        torso,
        Vec3::new(0.0, 0.15 * extent.y, 0.46 * extent.z),
        (2, tail_len),
    );
    armour(m, torso, extent);
}

/// Adds the torso and the four legs of a quadruped, with tall pillar legs (`1.5 * LEG`) for a titan
/// browser. Returns the torso and its extent.
///
/// The leg is [`LEG`] of the body length, which grows as `mass^(1/3)`: mammal limb bones scale
/// close to geometric similarity (Alexander and others 1979, "Allometry of the limb bones of mammals
/// from shrews (Sorex) to elephant (Loxodonta)", <https://doi.org/10.1111/j.1469-7998.1979.tb03964.x>),
/// so a 75 kg grazer stands about 0.8 m at the hip and a 500 kg one about 1.5 m.
fn stand(m: &mut Make, browser: bool) -> (u8, Vec3) {
    let s = m.t.size;
    let leg = LEG * s * m.t.reach * if browser { 1.5 } else { 1.0 } * m.vary(0.1);
    let girth = m.t.round * m.t.hide;
    let height = if browser { 0.42 } else { 0.34 } * s * girth;
    let extent = Vec3::new(0.36 * s * girth, height, 0.7 * s * m.vary(0.1));
    let radius = (if browser { 0.052 } else { 0.045 }) * s * m.t.thick;
    let body = piece(PartKind::Torso, ShapeKind::Ball, extent);
    let hip = seat(
        &body,
        Vec3::new(0.3 * extent.x, -0.3 * extent.y, 0.34 * extent.z),
        radius,
    );
    let torso = m
        .kit
        .add(None, body.at(Vec3::Y * (stance(leg, 0.0) - hip.y)));
    for (rank, z, knee) in [(0, -1.0, Knee::Back), (1, 1.0, Knee::Forward)] {
        let spec = LegSpec {
            hip: hip * Vec3::new(1.0, 1.0, z),
            bones: [0.48 * leg, 0.52 * leg, 0.0],
            radius,
            knee,
            out: Vec3::ZERO,
            rank,
            shapes: [ShapeKind::Taper; 2],
        };
        legs(&mut m.kit, torso, &spec);
    }
    (torso, extent)
}

/// Builds a two-legged runner with back-bending ankles and a tail for balance.
fn runner(m: &mut Make) {
    let s = m.t.size;
    let leg = 0.75 * s * m.t.reach * m.vary(0.1);
    let extent = Vec3::new(0.32, 0.34 * m.t.hide, 0.55 / m.t.round) * s * m.t.round;
    let torso = biped(m, extent, (leg, [0.36, 0.38, 0.26], 0.05 * s * m.t.thick));
    let socket = Vec3::new(0.0, 0.3 * extent.y, -0.4 * extent.z);
    let dir = Vec3::new(0.0, 1.0, -0.6);
    let (tip, link) = neck(m, torso, (socket, dir), (2, 0.4 * s * m.t.append));
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.26 * s);
    let root = Vec3::new(0.0, 0.0, 0.45 * extent.z);
    tail(m, torso, root, (2, 0.7 * s * m.t.append));
}

/// Builds a tall flightless terror-bird strider when the air or mass cannot carry a flyer.
fn strider(m: &mut Make) {
    let s = m.t.size;
    let leg = 0.95 * s * m.t.reach * m.vary(0.1);
    let extent = Vec3::new(0.34, 0.38 * m.t.hide, 0.58 / m.t.round) * s * m.t.round;
    let torso = biped(m, extent, (leg, [0.34, 0.4, 0.26], 0.055 * s * m.t.thick));
    let socket = Vec3::new(0.0, 0.32 * extent.y, -0.38 * extent.z);
    let (tip, link) = neck(
        m,
        torso,
        (socket, Vec3::new(0.0, 1.0, -0.3)),
        (2, 0.55 * s * m.t.append),
    );
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.32 * s);
    tail(
        m,
        torso,
        Vec3::new(0.0, 0.1 * extent.y, 0.45 * extent.z),
        (1, 0.35 * s * m.t.append),
    );
}

/// Builds a flyer: a light torso, a pair of wings, short legs and a fanned tail.
fn flyer(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::new(0.28 * m.t.round, 0.26 * m.t.round, 0.6) * s;
    let torso = biped(
        m,
        extent,
        (
            0.4 * s * m.t.reach,
            [0.45, 0.55, 0.0],
            0.035 * s * m.t.thick,
        ),
    );
    wings(m, torso, extent);
    let socket = Vec3::new(0.0, 0.2 * extent.y, -0.45 * extent.z);
    let (tip, link) = neck(m, torso, (socket, Vec3::new(0.0, 0.5, -1.0)), (1, 0.15 * s));
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.24 * s);
    if m.kit.fits(1) {
        let fan = piece(
            PartKind::Tail,
            ShapeKind::Fan,
            Vec3::new(0.02, 0.35, 0.3) * s,
        )
        .at(Vec3::new(0.0, 0.0, 0.45 * extent.z))
        .turned(flat(Vec3::new(0.0, -0.15, 1.0)));
        m.kit.add(Some(torso), fan);
    }
}

/// Adds a torso of extent `extent` on a pair of back-kneed legs of length `leg`, split into bones by
/// `split`, of thigh radius `radius`. Returns the torso.
pub(super) fn biped(m: &mut Make, extent: Vec3, (leg, split, radius): (f32, [f32; 3], f32)) -> u8 {
    let body = piece(PartKind::Torso, ShapeKind::Ball, extent);
    let hip = seat(
        &body,
        Vec3::new(0.28 * extent.x, -0.3 * extent.y, 0.1 * extent.z),
        radius,
    );
    let torso = m
        .kit
        .add(None, body.at(Vec3::Y * (stance(leg, 0.0) - hip.y)));
    let spec = LegSpec {
        hip,
        bones: split.map(|f| f * leg),
        radius,
        knee: Knee::Back,
        out: Vec3::ZERO,
        rank: 0,
        shapes: [ShapeKind::Taper; 2],
    };
    legs(&mut m.kit, torso, &spec);
    torso
}

/// Adds a pair of wings at the shoulders of a torso of extent `extent`, spanning what the air allows.
fn wings(m: &mut Make, torso: u8, extent: Vec3) {
    let span = m.t.wing.unwrap_or(1.2) * m.t.size;
    let length = 0.5 * span - 0.4 * extent.x;
    let wing = piece(
        PartKind::Wing,
        ShapeKind::Fan,
        Vec3::new(0.02, length, 0.45 * m.t.size),
    )
    .at(Vec3::new(0.4 * extent.x, 0.25 * extent.y, -0.1 * extent.z))
    .turned(aim(Vec3::X));
    let parts = m.kit.pair([torso; 2], wing);
    let rate = (4.5 / span.max(0.01).sqrt()).clamp(1.0, 30.0);
    m.kit.body.wings = Some(Wings { parts, span, rate });
}
