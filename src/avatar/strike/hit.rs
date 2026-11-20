//! Hits: the swept tool against the bounds of a creature part, and the blow it lands.

use bevy::prelude::*;

use super::outcome::Outcome;
use super::tune::SWEET_FLOOR;
use crate::craft::build::{Blow, Intent};

/// The head of the tool that struck, as the sound and the harm read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tip {
    /// Young's modulus of the material of the head, in gigapascals.
    pub stiffness: f32,
    /// Mass of the head, in kilograms.
    pub mass: f32,
}

/// A blow that landed on a creature part. Harm to animals, the wear of the tool and the hitstop
/// read it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// Root of the body that struck.
    pub attacker: Entity,
    /// Id in the pack of the tool that struck.
    pub item: u32,
    /// Root of the animal struck.
    pub victim: Entity,
    /// The part struck.
    pub part: Entity,
    /// Where it struck, in render space.
    pub at: Vec3,
    /// Unit direction the tool moved at contact, in render space.
    pub along: Vec3,
    /// What arrived: the energy at the contact radius times the sweet spot, the contact width and
    /// area, the hardness and how it meets the target.
    pub blow: Blow,
    /// What the blow was meant to do.
    pub intent: Intent,
    /// The head of the tool that struck.
    pub tip: Tip,
    /// What the swing came to: clean, glanced off or stopped.
    pub outcome: Outcome,
}

/// A box in its own frame: a part's bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Centre of the box.
    pub centre: Vec3,
    /// Half its extents along each axis.
    pub half: Vec3,
}

/// Returns the share along the segment `from` → `to` at which it first enters `bounds` grown by
/// `pad` on every side, or nothing when it misses (the slab test,
/// https://en.wikipedia.org/wiki/Slab_method).
pub fn enter(from: Vec3, to: Vec3, bounds: &Bounds, pad: Vec3) -> Option<f32> {
    let (low, high) = (
        bounds.centre - bounds.half - pad,
        bounds.centre + bounds.half + pad,
    );
    let way = to - from;
    let (mut near, mut far) = (0.0_f32, 1.0_f32);
    for axis in 0..3 {
        let (o, d) = (from[axis], way[axis]);
        if d.abs() < 1e-9 {
            if o < low[axis] || o > high[axis] {
                return None;
            }
            continue;
        }
        let (a, b) = ((low[axis] - o) / d, (high[axis] - o) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

/// Returns the share of the energy a blow keeps when it lands `radius` metres from the hand on a
/// tool whose head sits at `head` metres: all of it at the head, falling to [`SWEET_FLOOR`] at the
/// grip and past the tip.
pub fn sweet(radius: f32, head: f32) -> f32 {
    let head = head.max(f32::EPSILON);
    (1.0 - (radius - head).abs() / head).max(SWEET_FLOOR)
}

/// Returns the distance from `grip` along the tool to the point of the blade `grip` → `tip` nearest
/// to `point`.
pub fn radius(grip: Vec3, tip: Vec3, point: Vec3) -> f32 {
    let blade = tip - grip;
    let length = blade.length().max(f32::EPSILON);
    (point - grip).dot(blade / length).clamp(0.0, length)
}

/// Returns where the head path `was` → `now` (each a grip and a tip, in render space) first
/// touches a part of `bounds` placed by `part`, or else where the blade `now` lies in it, or
/// nothing. The paths are tested in the frame of the part, padded by `pad` metres.
pub fn touch(
    was: [Vec3; 2],
    now: [Vec3; 2],
    part: &GlobalTransform,
    bounds: &Bounds,
    pad: f32,
) -> Option<Vec3> {
    let inverse = part.affine().inverse();
    let scale = part.scale().abs().max(Vec3::splat(1e-3));
    let local = |p: Vec3| inverse.transform_point3(p);
    let pad = Vec3::splat(pad) / scale;
    let head = enter(local(was[1]), local(now[1]), bounds, pad).map(|s| was[1].lerp(now[1], s));
    head.or_else(|| {
        enter(local(now[0]), local(now[1]), bounds, pad).map(|s| now[0].lerp(now[1], s))
    })
}
