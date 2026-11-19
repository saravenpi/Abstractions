//! Colliders of the drawn shapes no single box can follow: cones, boulders and the upright ends of roofs.
//! Each is a cone, a pyramid, a box or a short stack of level boxes within a hand of the drawn face. Also
//! the one collider all the solid parts of a chunk become.

use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::brush::{Build, Part, Shape};
use super::plates::THICK;

/// Greatest distance in meters a stack may leave between a drawn face and its collider.
const GAP: f32 = 0.28;

/// Height in meters under which a cone or a roof end needs no collider of its own: what carries it is
/// within reach of all of it.
const LOW: f32 = 0.36;

/// Returns how many level steps follow a slanted face that spans `run` across and `rise` up within [`GAP`].
/// A point of the face is at most `run * rise / (2 * steps * (run + rise))` from the step beside it or
/// the step under it.
fn steps(run: f32, rise: f32) -> u32 {
    let needed = run * rise / ((run + rise) * 2.0 * GAP);
    (needed.ceil() as u32).clamp(1, 8)
}

/// Returns the first and the last share of the way along `profile`, a list of shares with the height at
/// each, at which it is at least `level` high.
fn span(profile: &[(f32, f32)], level: f32) -> Option<(f32, f32)> {
    let height = |k: f32| {
        let after = profile.iter().position(|p| p.0 >= k).unwrap_or(0).max(1);
        let (a, b) = (profile[after - 1], profile[after]);
        a.1 + (b.1 - a.1) * (k - a.0) / (b.0 - a.0).max(1e-6)
    };
    let kept = (0..=64)
        .map(|i| i as f32 / 64.0)
        .filter(|k| height(*k) >= level);
    kept.fold(None, |span, k| {
        Some((span.map_or(k, |s: (f32, f32)| s.0), k))
    })
}

/// Returns the compound collider of `parts`, or nothing when there is no solid part.
pub fn compound(parts: &[Part]) -> Option<Collider> {
    let shapes = parts.iter().filter_map(|p| {
        let shape = match p.shape {
            Shape::Cuboid(size) => Collider::cuboid(size.x, size.y, size.z),
            Shape::Cylinder(radius, height) => Collider::cylinder(radius, height),
            Shape::Cone(radius, height) => Collider::cone(radius, height),
            Shape::Hull(points) => Collider::convex_hull(&points)?,
        };
        Some((p.pos, p.rot, shape))
    });
    (!parts.is_empty()).then(|| Collider::compound(shapes.collect()))
}

impl Build<'_> {
    /// Adds the collider of the cone drawn on the chunk point `at` with the radius and height `size`: a cone
    /// halfway between its corners and its faces for five sides and more, the pyramid itself for four.
    /// Nothing for a cone under [`LOW`].
    pub fn cone_body(&mut self, at: Vec3, size: Vec2, sides: u32) {
        let (radius, height) = (size.x, size.y);
        if height < LOW {
            return;
        }
        let fit = (1.0 + (PI / sides as f32).cos()) * 0.5;
        let tip = Vec3::Y * height;
        let (pos, shape) = match sides {
            4 => {
                let (x, z) = (Vec3::X * radius, Vec3::Z * radius);
                (at, Shape::Hull([x, z, -x, -z, tip, tip]))
            }
            _ => (at + tip * 0.5, Shape::Cone(radius * fit, height)),
        };
        let rot = Quat::IDENTITY;
        self.parts.push(Part { pos, rot, shape });
    }

    /// Adds the collider of a boulder drawn around the chunk point `at` with the half sizes `radii`.
    pub fn boulder(&mut self, at: Vec3, radii: Vec3) {
        self.parts.push(Part {
            pos: at,
            rot: Quat::IDENTITY,
            shape: Shape::Cuboid(radii * 1.6),
        });
    }

    /// Adds the collider of an upright sheet that stands on the level local edge `from..to` and is as high
    /// along it as `profile` says, in shares of the way with the height at each: level boxes [`THICK`]
    /// thick, each as long as the sheet is wide at its middle height. Nothing for a sheet under [`LOW`].
    pub fn end_stack(&mut self, from: Vec3, to: Vec3, profile: &[(f32, f32)]) {
        let top = profile.iter().map(|p| p.1).fold(0.0, f32::max);
        if top <= LOW {
            return;
        }
        let along = to - from;
        let thick = along.normalize().cross(Vec3::Y) * THICK * 0.5;
        let count = steps(along.length() * 0.5, top);
        for i in 0..count {
            let (low, high) = (
                top * i as f32 / count as f32,
                top * (i + 1) as f32 / count as f32,
            );
            let Some((first, last)) = span(profile, (low + high) * 0.5) else {
                continue;
            };
            let p = from + along * first - thick + Vec3::Y * low;
            let q = from + along * last + thick + Vec3::Y * high;
            self.collider(p.min(q), p.max(q));
        }
    }
}
