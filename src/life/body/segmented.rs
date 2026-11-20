//! Segmented archetypes: the many-legged crawler, the legless worm and the shelled hexapod.

use bevy::math::{Quat, Vec3};

use super::frame::aim;
use super::head::head;
use super::leg::{LegSpec, legs, stance};
use super::make::Make;
use super::piece::piece;
use super::{Chain, FeatureKind, Knee, Part, PartKind, ShapeKind};

/// Number of segmented archetypes.
pub const COUNT: u8 = 3;

/// Most parts of a many-legged crawler: a head, seven segments, fourteen legs of two bones, eyes and feelers.
pub const CRAWLER_PARTS: usize = 40;

/// Largest bend between two segments, in radians.
const BEND: f32 = 0.35;

/// Builds segmented archetype `archetype`: 0 many-legged crawler, 1 legless worm, 2 shelled hexapod.
pub fn build(m: &mut Make, archetype: u8) {
    match archetype % COUNT {
        0 => crawler(m),
        1 => worm(m),
        _ => hexapod(m),
    }
}

/// Adds a spine of `count` hanging segments laid backward from the front end at `front`, each `link` long
/// and `width` wide and thinning toward the tail, and returns it.
fn spine(m: &mut Make, front: Vec3, (count, link, width): (usize, f32, f32)) -> Chain {
    let segment = |i: usize| {
        let scale = 1.0 - 0.4 * i as f32 / count as f32;
        let size = Vec3::new(width * scale, link, width * scale * 0.8);
        let (at, rest) = match i {
            0 => (front, aim(Vec3::Z)),
            _ => (Vec3::new(0.0, -link, 0.0), Quat::IDENTITY),
        };
        let kind = if i == 0 {
            PartKind::Torso
        } else {
            PartKind::Segment
        };
        piece(kind, ShapeKind::Taper, size)
            .at(at)
            .turned(rest)
            .of(0, 0.85)
    };
    let links: Vec<Part> = (0..count).map(segment).collect();
    m.kit.chain(None, &links, BEND)
}

/// Builds a crawler with one pair of thin, splayed legs on each of its 3 to 7 segments.
fn crawler(m: &mut Make) {
    let s = m.t.size;
    let pairs = 3 + m.rng.below(5) as usize;
    let link = s / pairs as f32;
    let width = 0.16 * s * m.t.round;
    let leg = 0.4 * s * m.t.reach * m.vary(0.1);
    let splay = 0.55 * leg;
    let front = Vec3::new(0.0, stance(leg, splay), -0.5 * s);
    let chain = spine(m, front, (pairs, link, width));
    for (rank, &segment) in chain.parts.iter().enumerate() {
        let spec = LegSpec {
            hip: Vec3::new(0.4 * width, -0.5 * link, 0.0),
            bones: [0.45 * leg, 0.55 * leg, 0.0],
            radius: 0.03 * s * m.t.thick,
            knee: Knee::Out,
            out: Vec3::X * splay,
            rank: rank as u8,
            shapes: [ShapeKind::Spike; 2],
        };
        legs(&mut m.kit, segment, &spec);
    }
    front_end(m, chain, 0.3 * s);
}

/// Builds a legless worm of 6 to 10 segments that slithers.
fn worm(m: &mut Make) {
    let s = m.t.size;
    let count = 6 + m.rng.below(5) as usize;
    let width = 0.12 * s * m.t.round * m.t.hide;
    let chain = spine(
        m,
        Vec3::new(0.0, 0.45 * width, -0.5 * s),
        (count, s / count as f32, width),
    );
    front_end(m, chain, 0.2 * s);
}

/// Adds a head of length `length` at the front of `spine`, with feelers if they fit, and keeps the spine.
fn front_end(m: &mut Make, spine: Chain, length: f32) {
    let root = spine.parts[0];
    let index = head(m, root, Vec3::ZERO, length);
    feelers(m, index, length);
    m.kit.body.spine = Some(spine);
}

/// Builds a hexapod under a hard shell: three pairs of splayed legs from the sides of its shell, a head
/// and feelers.
fn hexapod(m: &mut Make) {
    let s = m.t.size;
    let extent = Vec3::new(0.5 * s * m.t.round, 0.3 * s * m.t.hide, 0.75 * s);
    let leg = 0.55 * s * m.t.reach * m.vary(0.1);
    let splay = 0.5 * leg;
    let hip = Vec3::new(0.38 * extent.x, 0.0, 0.0);
    let shell =
        piece(PartKind::Torso, ShapeKind::Ball, extent).at(Vec3::Y * (stance(leg, splay) - hip.y));
    let torso = m.kit.add(None, shell);
    for (rank, z) in [-0.3, 0.0, 0.3].into_iter().enumerate() {
        let spec = LegSpec {
            hip: hip + Vec3::Z * z * extent.z,
            bones: [0.4 * leg, 0.6 * leg, 0.0],
            radius: 0.035 * s * m.t.thick,
            knee: Knee::Out,
            out: Vec3::X * splay,
            rank: rank as u8,
            shapes: [ShapeKind::Taper, ShapeKind::Spike],
        };
        legs(&mut m.kit, torso, &spec);
    }
    m.kit.feature(FeatureKind::Plate, extent.z, 0);
    let index = head(m, torso, Vec3::new(0.0, 0.0, -0.5 * extent.z), 0.3 * s);
    feelers(m, index, 0.3 * s);
}

/// Adds a pair of feelers on a head of length `length`, if they fit.
fn feelers(m: &mut Make, head: u8, length: f32) {
    if !m.kit.fits(2) {
        return;
    }
    let reach = 1.4 * length * m.t.append;
    let feeler = piece(
        PartKind::Antenna,
        ShapeKind::Spike,
        Vec3::new(0.08 * length, reach, 0.08 * length),
    )
    .at(Vec3::new(0.2 * length, 0.25 * length, -0.35 * length))
    .turned(aim(Vec3::new(0.35, 0.6, -1.0)));
    m.kit.pair([head; 2], feeler);
}
