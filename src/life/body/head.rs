//! Heads shaped by the guild: a flat wide mouth for grazers, jaws and horns for hunters, sieves for
//! filter feeders, beaks for seed and fruit eaters, glands for decomposers. Eyes grow where light is dim.

use std::f32::consts::FRAC_PI_2;

use bevy::math::{Quat, Vec3};

use super::frame::{aim, frame};
use super::make::Make;
use super::piece::piece;
use super::{FeatureKind, PartKind, ShapeKind};
use crate::life::Guild;

/// Adds a head of length `length` under `parent` at `socket`, level in the body frame, with its eyes and
/// mouth parts. Returns the index of the head.
pub fn head(m: &mut Make, parent: u8, socket: Vec3, length: f32) -> u8 {
    let (_, turn) = frame(&m.kit.body.parts, usize::from(parent));
    let (shape, shape_of) = match m.guild {
        Guild::Grazer => (ShapeKind::Block, Vec3::new(0.85, 0.5, 1.0)),
        Guild::Browser => (ShapeKind::Ball, Vec3::new(0.5, 0.5, 1.0)),
        Guild::Hunter | Guild::Ambusher => (ShapeKind::Ball, Vec3::new(0.75, 0.7, 1.0)),
        _ => (ShapeKind::Ball, Vec3::new(0.7, 0.7, 0.85)),
    };
    let size = shape_of * length;
    let part = piece(PartKind::Head, shape, size)
        .at(socket)
        .turned(turn.inverse());
    let index = m.kit.add(Some(parent), part);
    eyes(m, index, size);
    mouth(m, index, size);
    index
}

/// Adds a pair of eyes on the head `head` of extent `size`, if they fit.
fn eyes(m: &mut Make, head: u8, size: Vec3) {
    if !m.kit.fits(2) {
        return;
    }
    let radius = (m.t.eye * size.y).min(0.3 * size.x);
    let socket = Vec3::new(0.36 * size.x, 0.2 * size.y, -0.3 * size.z);
    let eye = piece(PartKind::Eye, ShapeKind::Ball, Vec3::splat(2.0 * radius)).at(socket);
    m.kit.pair([head; 2], eye.of(2, 1.0));
}

/// Adds what the guild puts on a head of extent `size`.
fn mouth(m: &mut Make, head: u8, size: Vec3) {
    let guild = m.guild;
    match guild {
        Guild::Grazer if m.chance(0.55) => horns(m, head, size, 0.95),
        Guild::Browser if m.chance(0.45) => horns(m, head, size, 0.85),
        Guild::Hunter => {
            jaw(m, head, size);
            horns(m, head, size, 1.0);
        }
        Guild::Ambusher => {
            jaw(m, head, size);
            if m.chance(0.5) {
                horns(m, head, size, 0.9);
            }
        }
        Guild::Filter => sieve(m, head, size),
        Guild::Nectar => beak(m, head, size, 1.1),
        Guild::Frugivore | Guild::Granivore | Guild::Scavenger => beak(m, head, size, 0.5),
        Guild::Decomposer | Guild::Parasite => gland(m, head, size),
        _ => {}
    }
}

/// Adds a lower jaw hanging forward from under the front of the head.
fn jaw(m: &mut Make, head: u8, size: Vec3) {
    if !m.kit.fits(1) {
        return;
    }
    let length = 0.75 * size.z;
    let part = piece(
        PartKind::Jaw,
        ShapeKind::Taper,
        Vec3::new(0.7 * size.x, length, 0.35 * size.y),
    )
    .at(Vec3::new(0.0, -0.25 * size.y, -0.2 * size.z))
    .turned(aim(Vec3::new(0.0, -0.3, -1.0)));
    m.kit.add(Some(head), part.of(0, 0.6));
    m.kit.feature(FeatureKind::Jaw, length, 0);
}

/// Adds horns, swept crests, forward bull horns, a nasal boss horn or forward mandibles on `head` of
/// extent `size`, `scale` times the plain length, if they fit.
pub fn horns(m: &mut Make, head: u8, size: Vec3, scale: f32) {
    const STYLES: [(f32, f32, [f32; 3], [f32; 3]); 4] = [
        (0.22, 1.0, [0.32, 0.35, -0.15], [0.75, 0.75, -0.65]),
        (0.22, 1.0, [0.28, 0.42, 0.05], [0.35, 0.85, 0.75]),
        (0.26, 1.15, [0.0, 0.25, -0.38], [0.0, 0.95, -0.55]),
        (0.22, 1.0, [0.32, -0.15, -0.35], [0.45, -0.2, -1.0]),
    ];
    let style = m.rng.below(4) as usize;
    let single = style == 2;
    let length = scale * size.y * m.t.append * m.vary(0.2) * STYLES[style].1;
    if !m.kit.fits(if single { 1 } else { 2 }) {
        return;
    }
    let (w, _, at, dir) = STYLES[style];
    let extent = Vec3::new(w * size.x, length, w * size.x);
    let horn = piece(PartKind::Horn, ShapeKind::Spike, extent)
        .at(Vec3::from_array(at) * size)
        .turned(aim(Vec3::from_array(dir)))
        .of(1, 1.0);
    if single {
        m.kit.add(Some(head), horn);
    } else {
        m.kit.pair([head; 2], horn);
    }
    m.kit.feature(FeatureKind::Horn, length, 1);
}

/// Adds a beak pointing forward from the front of the head, `scale` times the head length.
fn beak(m: &mut Make, head: u8, size: Vec3, scale: f32) {
    if !m.kit.fits(1) {
        return;
    }
    let length = scale * size.z;
    let part = piece(
        PartKind::Jaw,
        ShapeKind::Spike,
        Vec3::new(0.4 * size.x, length, 0.35 * size.y),
    )
    .at(Vec3::new(0.0, -0.1 * size.y, -0.4 * size.z))
    .turned(aim(Vec3::new(0.0, -0.15, -1.0)));
    m.kit.add(Some(head), part.of(1, 1.0));
    m.kit.feature(FeatureKind::Jaw, length, 1);
}

/// Adds a sieve hanging under the mouth: a fan of fringes that strains food from water or air.
fn sieve(m: &mut Make, head: u8, size: Vec3) {
    if !m.kit.fits(1) {
        return;
    }
    let part = piece(
        PartKind::Jaw,
        ShapeKind::Fan,
        Vec3::new(0.02, 0.6 * size.y, 0.8 * size.x),
    )
    .at(Vec3::new(0.0, -0.35 * size.y, -0.35 * size.z))
    .turned(aim(Vec3::new(0.0, -1.0, -0.4)) * Quat::from_rotation_y(FRAC_PI_2));
    m.kit.add(Some(head), part.of(1, 1.0));
}

/// Adds a gland on top of the head.
fn gland(m: &mut Make, head: u8, size: Vec3) {
    if !m.kit.fits(1) {
        return;
    }
    let radius = 0.3 * size.x;
    let part = piece(PartKind::Blob, ShapeKind::Ball, Vec3::splat(2.0 * radius)).at(Vec3::new(
        0.0,
        0.45 * size.y,
        0.1 * size.z,
    ));
    m.kit.add(Some(head), part.of(2, 1.0));
    m.kit.feature(FeatureKind::Gland, 2.0 * radius, 2);
}
