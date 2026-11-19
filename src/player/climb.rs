//! Step-up: a body on foot that walks into a low step is lifted onto it.
//!
//! Move and slide stops the capsule at the riser of a step. When that happens on the ground,
//! the capsule is cast up, forward by the motion it lost, then down onto the top of the step.
//! The eyes keep the jump and let it go over [`EASE`] seconds, so the view never snaps.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::ground::{MIN_GROUND_NORMAL, SKIN};
use super::{BODY_LENGTH, BODY_RADIUS};

/// Highest step the body climbs, in metres above its soles.
pub(super) const STEP_UP: f32 = 0.35;

/// Shortest forward move of a step-up, so the soles end over the step and not on its edge.
const REACH: f32 = 0.2;

/// Lost motion below which there is nothing to climb, in metres.
const MIN_LOST: f32 = 1.0e-3;

/// Seconds over which the eyes catch up with a step.
const EASE: f32 = 0.12;

/// Distance from the capsule centre down to its soles.
const SOLES: f32 = BODY_LENGTH * 0.5 + BODY_RADIUS;

/// What a step-up casts with: the capsule of a body, its pose and what it ignores.
pub(super) struct Reach<'a, 'w, 's> {
    /// The scene queries.
    query: &'a SpatialQuery<'w, 's>,
    /// The capsule of the body.
    collider: &'a Collider,
    /// Orientation of the capsule.
    rotation: Quat,
    /// The body itself, which the casts ignore.
    filter: &'a SpatialQueryFilter,
    /// Turns the upright frame of the body into render space.
    tilt: Quat,
}

impl<'a, 'w, 's> Reach<'a, 'w, 's> {
    /// Returns the casts of the capsule `collider` posed at `pose`, ignoring what `filter`
    /// leaves out, for a body whose upright frame `tilt` turns into render space.
    pub(super) fn of(
        query: &'a SpatialQuery<'w, 's>,
        (collider, pose): (&'a Collider, &Transform),
        filter: &'a SpatialQueryFilter,
        tilt: Quat,
    ) -> Self {
        Self {
            query,
            collider,
            rotation: pose.rotation,
            filter,
            tilt,
        }
    }

    /// Casts the capsule from `from` along `dir` for at most `max` metres.
    fn cast(&self, from: Vec3, dir: Vec3, max: f32) -> Option<ShapeHitData> {
        let dir = Dir3::new(dir).ok()?;
        let config = ShapeCastConfig::from_max_distance(max);
        self.query.cast_shape(
            self.collider,
            from,
            self.rotation,
            dir,
            &config,
            self.filter,
        )
    }

    /// Returns how far the capsule moves from `from` along `dir`, at most `max`, and keeps
    /// [`SKIN`] from what it meets.
    fn free(&self, from: Vec3, dir: Vec3, max: f32) -> f32 {
        self.cast(from, dir, max)
            .map_or(max, |hit| (hit.distance - SKIN).max(0.0))
    }

    /// Returns where the body stands after climbing from `at`, where move and slide left it,
    /// when it was wished to move by `wished` from `from` and lost motion on the way: on the
    /// top of a step at most [`STEP_UP`] above its soles that is flat enough to stand on.
    ///
    /// All vectors are in render space.
    pub(super) fn climb(&self, (from, at): (Vec3, Vec3), wished: Vec3) -> Option<Vec3> {
        let up = self.tilt * Vec3::Y;
        let went = at - from;
        let lost = (wished - went).reject_from_normalized(up);
        if lost.length() < MIN_LOST {
            return None;
        }
        let rise = self.free(at, up, STEP_UP);
        let raised = at + up * rise;
        let ahead = lost.normalize() * self.free(raised, lost, lost.length().max(REACH));
        let hit = self
            .cast(raised + ahead, -up, rise)
            .filter(|_| ahead.length() > MIN_LOST)?;
        let top = up.dot(hit.point1 - at) + SOLES;
        let flat = (self.tilt.inverse() * hit.normal1).y >= MIN_GROUND_NORMAL;
        let drop = (hit.distance - SKIN).max(0.0);
        (flat && top > 0.0 && top <= STEP_UP).then_some(raised + ahead - up * drop)
    }
}

/// Returns the lag of the eyes behind the body `dt` seconds later. It closes at a steady speed
/// that lets a full step go in [`EASE`] seconds, and faster when the lag is larger, so a run up
/// a long stair never leaves the eyes far behind.
pub(super) fn ease(lag: Vec3, dt: f32) -> Vec3 {
    let speed = lag.length().max(STEP_UP) / EASE;
    lag - lag.clamp_length_max(speed * dt)
}
