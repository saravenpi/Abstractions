//! Sinking limb roots into the body: a leg, neck, tail, wing or fin is joined inside the wall of the
//! part it grows from, as a thigh sits in the haunch, not glued to the outline.
//!
//! Depths are read on the drawn low-poly shapes of `shape`: a ball is two rings of radius 0.866 at
//! half its height, closed by cones to its poles, and the flat sides between ring corners lie a
//! little further in.

use std::f32::consts::PI;

use bevy::math::{Vec2, Vec3};

use super::shape::sides;
use super::{Part, PartKind, ShapeKind};

/// Steps of the search for the deepest place a socket needs.
const STEPS: u32 = 16;

/// Returns true for the kinds of part that grow from the body as a limb: legs, necks, tails, wings
/// and fins.
pub fn limb(kind: PartKind) -> bool {
    matches!(
        kind,
        PartKind::Thigh | PartKind::Neck | PartKind::Tail | PartKind::Wing | PartKind::Fin
    )
}

/// Returns the radius in metres of the root of `part` where it joins its parent.
pub fn root(part: &Part) -> f32 {
    match part.shape {
        ShapeKind::Fan => 0.25 * part.size.z,
        _ => 0.5 * part.size.x.min(part.size.z),
    }
}

/// Returns the share of its outer radius at which the flat sides of a ring of `part` lie.
fn flat(part: &Part) -> f32 {
    (PI / sides(part) as f32).cos()
}

/// Returns how deep in metres point `p` of the frame of `part` lies inside its drawn shape, negative
/// outside. Fans and spikes hold nothing.
pub fn depth(part: &Part, p: Vec3) -> f32 {
    let half = part.size * 0.5;
    match part.shape {
        ShapeKind::Ball => {
            let n = p / half.max(Vec3::splat(1e-4));
            let ring = (1.0 - n.y * n.y).max(0.0).sqrt() * flat(part);
            (ring - Vec2::new(n.x, n.z).length()) * half.x.min(half.z)
        }
        ShapeKind::Block => (half - p.abs()).min_element(),
        ShapeKind::Taper => {
            let u = (-p.y / part.size.y.max(1e-4)).clamp(0.0, 1.0);
            let wide = flat(part) * (1.0 + (part.taper - 1.0) * u);
            let n = Vec2::new(p.x / half.x.max(1e-4), p.z / half.z.max(1e-4));
            let side = (wide - n.length()) * half.x.min(half.z);
            side.min(-p.y).min(p.y + part.size.y)
        }
        ShapeKind::Spike | ShapeKind::Fan => -1.0,
    }
}

/// Returns the point of `part` its sockets are pulled towards: its centre, or the axis of a taper.
fn core(part: &Part, p: Vec3) -> Vec3 {
    match part.shape {
        ShapeKind::Taper => Vec3::new(0.0, p.y.clamp(-part.size.y, 0.0), 0.0),
        _ => Vec3::ZERO,
    }
}

/// Returns `socket` in the frame of `parent` moved straight in towards its core until it lies `radius`
/// deep, or as deep as the core when the parent is thinner than that.
pub fn sink(parent: &Part, socket: Vec3, radius: f32) -> Vec3 {
    let centre = core(parent, socket);
    let want = radius.min(depth(parent, centre));
    if want <= 0.0 || depth(parent, socket) >= want {
        return socket;
    }
    let (mut inside, mut outside) = (0.0_f32, 1.0_f32);
    for _ in 0..STEPS {
        let t = 0.5 * (inside + outside);
        if depth(parent, centre + (socket - centre) * t) >= want {
            inside = t;
        } else {
            outside = t;
        }
    }
    centre + (socket - centre) * inside
}
