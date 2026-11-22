//! Parts many archetypes share: necks, tails, dorsal sails, crests and armour plates on the back.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};

use super::frame::aim;
use super::make::Make;
use super::piece::piece;
use super::{FeatureKind, Part, PartKind, ShapeKind};

/// Largest bend per joint of a neck or a tail, in radians.
const BEND: f32 = 0.5;

/// Returns `count` hanging links of `kind`, each `length` long and `width` wide, the first at `socket`
/// pointing along `dir` in the parent frame and each next one bent by `curl` radians about X.
fn links(
    kind: PartKind,
    (socket, dir): (Vec3, Vec3),
    (count, length, width): (usize, f32, f32),
    curl: f32,
) -> Vec<Part> {
    let size = Vec3::new(width, length, width);
    (0..count)
        .map(|i| {
            let first = i == 0;
            let at = if first {
                socket
            } else {
                Vec3::new(0.0, -length, 0.0)
            };
            let rest = if first {
                aim(dir)
            } else {
                Quat::from_rotation_x(curl)
            };
            let scale = 1.0 - 0.15 * i as f32;
            piece(kind, ShapeKind::Taper, size * Vec3::new(scale, 1.0, scale))
                .at(at)
                .turned(rest)
                .of(0, 0.8)
        })
        .collect()
}

/// Adds a neck of `count` links and total `length` under `parent`, from `socket` along `dir`.
/// Returns the index of its last link and the length of one link.
pub fn neck(
    m: &mut Make,
    parent: u8,
    (socket, dir): (Vec3, Vec3),
    (count, length): (usize, f32),
) -> (u8, f32) {
    let link = length / count as f32;
    let width = (0.11 * m.t.size * m.t.hide).min(0.8 * link.max(0.05));
    let parts = links(PartKind::Neck, (socket, dir), (count, link, width), -0.15);
    let chain = m.kit.chain(Some(parent), &parts, BEND);
    let tip = chain.parts.last().copied().unwrap_or(parent);
    m.kit.body.neck = Some(chain);
    if length > 0.3 * m.t.size {
        m.kit.feature(FeatureKind::Neck, length, 0);
    }
    (tip, link)
}

/// Adds a tail of up to `count` links and total `length` under `parent` from `socket`, if it fits, with
/// a pair of spikes or a club at the tip for some.
pub fn tail(m: &mut Make, parent: u8, socket: Vec3, (count, length): (usize, f32)) {
    let count = (1..=count).rev().find(|&c| m.kit.fits(c)).unwrap_or(0);
    if count == 0 {
        return;
    }
    let link = length / count as f32;
    let width = 0.09 * m.t.size;
    let parts = links(
        PartKind::Tail,
        (socket, Vec3::new(0.0, -0.4, 1.0)),
        (count, link, width),
        0.2,
    );
    let chain = m.kit.chain(Some(parent), &parts, BEND);
    let tip = chain.parts.last().copied().unwrap_or(parent);
    m.kit.body.tail = Some(chain);
    m.kit.feature(FeatureKind::Tail, length, 0);
    if m.kit.fits(2) && m.chance(0.25) {
        club(m, tip, link);
    }
}

/// Adds a pair of heavy lateral tail spikes or clubs near the end of tail link `tip` of length `link`.
pub fn club(m: &mut Make, tip: u8, link: f32) {
    if !m.kit.fits(2) {
        return;
    }
    let width = 0.09 * m.t.size;
    let spike = piece(
        PartKind::Horn,
        ShapeKind::Spike,
        Vec3::new(0.45 * width, 2.2 * width, 0.45 * width),
    )
    .at(Vec3::new(0.0, -0.8 * link, 0.0))
    .turned(aim(Vec3::new(1.0, 0.0, 0.3)));
    m.kit.pair([tip; 2], spike.of(1, 1.0));
    m.kit.feature(FeatureKind::Spike, 2.2 * width, 1);
}

/// Adds a pair of armour plates along the back of a torso of extent `extent`, for some and if they fit.
pub fn armour(m: &mut Make, torso: u8, extent: Vec3) {
    if m.chance(0.3) {
        plates(m, torso, extent);
    }
}

/// Adds a pair of heavy dorsal armour plates along the back of `torso` of extent `extent`, if they fit.
pub fn plates(m: &mut Make, torso: u8, extent: Vec3) {
    if !m.kit.fits(2) {
        return;
    }
    let size = Vec3::new(0.06 * extent.x, 0.45 * extent.y, 0.6 * extent.z);
    let plate = piece(PartKind::Plate, ShapeKind::Block, size)
        .at(Vec3::new(0.22 * extent.x, 0.45 * extent.y, 0.0))
        .turned(Quat::from_rotation_z(-0.35));
    m.kit.pair([torso; 2], plate.of(1, 1.0));
    m.kit.feature(FeatureKind::Plate, size.y, 1);
}

/// Adds a tall dorsal spine sail or ridge along the centre of `torso` of extent `extent`, if it fits.
pub fn sail(m: &mut Make, torso: u8, extent: Vec3, height: f32) {
    if !m.kit.fits(1) {
        return;
    }
    let size = Vec3::new(0.03 * m.t.size, height, 0.72 * extent.z);
    let fin = piece(PartKind::Plate, ShapeKind::Fan, size)
        .at(Vec3::new(0.0, 0.42 * extent.y, 0.0))
        .turned(aim(Vec3::Y));
    m.kit.add(Some(torso), fin.of(1, 1.0));
    m.kit.feature(FeatureKind::Plate, height, 1);
}

/// Returns the rest rotation that lays a fan flat across the body while it points along `dir`.
pub fn flat(dir: Vec3) -> Quat {
    aim(dir) * Quat::from_rotation_y(FRAC_PI_2)
}
