//! Skeletal archetypes: the quadruped grazer, the long-necked browser, the biped runner and the winged flyer.

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
pub const COUNT: u8 = 4;

/// Leg length of a quadruped over its body length scale.
const LEG: f32 = 0.75;

/// Builds skeletal archetype `archetype`: 0 quadruped grazer, 1 long-necked browser, 2 biped runner,
/// 3 winged flyer, which walks as a runner where the air cannot carry it.
pub fn build(m: &mut Make, archetype: u8) {
    match archetype % COUNT {
        0 => quadruped(m, false),
        1 => quadruped(m, true),
        2 => runner(m),
        _ if m.t.wing.is_some() => flyer(m),
        _ => runner(m),
    }
}

/// Builds a four-legged body, with a long neck and long legs for a browser.
fn quadruped(m: &mut Make, browser: bool) {
    let s = m.t.size;
    let (torso, extent) = stand(m, browser);
    let long = browser || m.guild == Guild::Browser;
    let (count, reach, dir) = match long {
        true => (3, 0.9 * s, Vec3::new(0.0, 1.0, -0.45)),
        false => (1, 0.3 * s, Vec3::new(0.0, 0.55, -1.0)),
    };
    let socket = Vec3::new(0.0, 0.2 * extent.y, -0.42 * extent.z);
    let (tip, link) = neck(m, torso, (socket, dir), (count, reach * m.t.append));
    head(m, tip, Vec3::new(0.0, -link, 0.0), 0.26 * s);
    tail(
        m,
        torso,
        Vec3::new(0.0, 0.15 * extent.y, 0.46 * extent.z),
        (2, 0.45 * s * m.t.append),
    );
    armour(m, torso, extent);
}

/// Adds the torso and the four legs of a quadruped, longer legged for a browser. Returns the torso and
/// its extent.
///
/// The leg is [`LEG`] of the body length, which grows as `mass^(1/3)`: mammal limb bones scale
/// close to geometric similarity (Alexander and others 1979, "Allometry of the limb bones of mammals
/// from shrews (Sorex) to elephant (Loxodonta)", <https://doi.org/10.1111/j.1469-7998.1979.tb03964.x>),
/// so a 75 kg grazer stands about 0.8 m at the hip and a 500 kg one about 1.5 m.
fn stand(m: &mut Make, browser: bool) -> (u8, Vec3) {
    let s = m.t.size;
    let leg = LEG * s * m.t.reach * if browser { 1.35 } else { 1.0 } * m.vary(0.1);
    let girth = m.t.round * m.t.hide;
    let extent = Vec3::new(0.36 * s * girth, 0.34 * s * girth, 0.7 * s * m.vary(0.1));
    let radius = 0.045 * s * m.t.thick;
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
fn biped(m: &mut Make, extent: Vec3, (leg, split, radius): (f32, [f32; 3], f32)) -> u8 {
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
