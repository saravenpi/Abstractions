//! Rigs: how the frame of a chunk takes the origin of a prop and the way the ground turns there, and how a
//! stone is tilted to the slope it lies on.

use bevy::prelude::*;

use super::Placement;

/// The two maps a chunk gives to place props rigidly in its frame. A chunk builder fills both fields with
/// closures over its own embedding.
///
/// Only the origin of a prop goes through the chart. Its geometry keeps its true shape and size in metres and is
/// turned as one piece, so nothing is stretched where the chart is squeezed.
#[derive(Clone, Copy)]
pub struct Rig<'a> {
    /// Returns the chunk frame position of a chunk space point: chart offsets `x` and `z`, height `y`.
    pub embed: &'a (dyn Fn(Vec3) -> Vec3 + Sync),
    /// Returns the rotation that carries directions at a chunk space point into the chunk frame: up to the
    /// local up, chart x to chart x.
    pub turn: &'a (dyn Fn(Vec3) -> Quat + Sync),
}

#[cfg(test)]
impl Rig<'static> {
    /// The rig of chunk space itself: nothing moves and nothing turns. It gives the legacy output, which a
    /// point by point warp can still carry onto the globe.
    pub const PLAIN: Self = Self {
        embed: &|p| p,
        turn: &|_| Quat::IDENTITY,
    };
}

/// Returns the turn that carries the local up onto the normal of ground of grade `grade`, rise over run
/// along x and z.
fn square(grade: Vec2) -> Quat {
    Quat::from_rotation_arc(Vec3::Y, Vec3::new(-grade.x, 1.0, -grade.y).normalize())
}

impl Rig<'_> {
    /// Returns the position and rotation, in the chunk frame, of a rigid body whose origin is chunk space
    /// point `at`.
    pub fn frame(&self, at: Vec3) -> (Vec3, Quat) {
        ((self.embed)(at), (self.turn)(at))
    }

    /// Returns the position and rotation of prop `p` in the chunk frame, its yaw included, and for a
    /// stone the tilt that sets it square to the ground it lies on.
    pub fn pose(&self, p: &Placement) -> (Vec3, Quat) {
        let (origin, turn) = self.frame(p.pos);
        let yaw = Quat::from_rotation_y(p.yaw);
        match p.lean == Vec2::ZERO {
            true => (origin, turn * yaw),
            false => (origin, turn * square(p.lean) * yaw),
        }
    }
}
