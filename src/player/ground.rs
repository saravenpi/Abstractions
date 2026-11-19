//! Ground detection with a shape cast down the local vertical, slope projection and ground snapping.
//!
//! Normals and velocities here are in the upright frame of the body, where +Y is the local up.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::{BODY_LENGTH, BODY_RADIUS};

/// Smallest surface normal height that counts as walkable ground (about 50 degrees).
pub(super) const MIN_GROUND_NORMAL: f32 = 0.64;

/// Longest distance below the capsule at which the body still sticks to the ground.
pub(super) const SNAP_DISTANCE: f32 = 0.4;

/// Upward speed above which the body is considered to be leaving the ground.
pub(super) const RISING_SPEED: f32 = 0.5;

/// Gap that is ignored when snapping, so the resting skin never causes motion.
pub(super) const SKIN: f32 = 0.03;

/// Ground found under the capsule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Ground {
    /// Surface normal at the contact.
    pub(super) normal: Vec3,
    /// Free distance between the capsule and the surface, measured straight down.
    pub(super) gap: f32,
}

/// Casts the capsule down the local vertical and returns the nearest surface within snap range.
///
/// When the capsule rests on an edge too steep to stand on, such as the nose of a stair step
/// it walks down from, the ground straight under its centre gives the normal instead.
/// `tilt` turns the upright frame of the body into render space.
pub(super) fn probe(
    query: &SpatialQuery,
    collider: &Collider,
    transform: &Transform,
    tilt: Quat,
    filter: &SpatialQueryFilter,
) -> Option<Ground> {
    let down = Dir3::new(tilt * Vec3::NEG_Y).unwrap_or(Dir3::NEG_Y);
    let hit = query.cast_shape(
        collider,
        transform.translation,
        transform.rotation,
        down,
        &ShapeCastConfig::from_max_distance(SNAP_DISTANCE),
        filter,
    )?;
    let normal = tilt.inverse() * hit.normal1;
    let normal = match normal.y >= MIN_GROUND_NORMAL {
        true => normal,
        false => {
            under(query, transform.translation, down, filter).map_or(normal, |n| tilt.inverse() * n)
        }
    };
    Some(Ground {
        normal,
        gap: hit.distance,
    })
}

/// Returns the normal of the ground straight down from the capsule centre at `at`, within
/// snap range of its soles.
fn under(query: &SpatialQuery, at: Vec3, down: Dir3, filter: &SpatialQueryFilter) -> Option<Vec3> {
    let reach = BODY_LENGTH * 0.5 + BODY_RADIUS + SNAP_DISTANCE;
    query
        .cast_ray(at, down, reach, true, filter)
        .map(|hit| hit.normal)
}

/// Keeps the ground only when it is flat enough to stand on and the body is not rising.
pub(super) fn walkable(ground: Option<Ground>, rising: bool) -> Option<Ground> {
    ground.filter(|ground| !rising && ground.normal.y >= MIN_GROUND_NORMAL)
}

/// Projects a horizontal velocity onto the ground plane and keeps its speed.
pub(super) fn slope_velocity(flat: Vec3, normal: Vec3) -> Vec3 {
    let along = flat - normal * flat.dot(normal);
    along.normalize_or_zero() * flat.length()
}

/// Returns the downward speed that closes a gap to the ground within one step.
pub(super) fn snap_velocity(gap: f32, dt: f32) -> f32 {
    -(gap - SKIN).max(0.0) / dt
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Slope projection keeps speed, climbs uphill and descends downhill.
    #[test]
    fn slope_projection_keeps_speed() {
        let normal = Vec3::new(-0.5, 0.866, 0.0).normalize();
        let uphill = slope_velocity(Vec3::new(5.0, 0.0, 0.0), normal);
        assert!((uphill.length() - 5.0).abs() < 1e-4);
        assert!(uphill.dot(normal).abs() < 1e-4);
        let downhill = slope_velocity(Vec3::new(-5.0, 0.0, 0.0), normal);
        assert!(uphill.y > 0.0 && downhill.y < 0.0);
        assert_eq!(slope_velocity(Vec3::ZERO, normal), Vec3::ZERO);
    }

    /// Steep ground and rising bodies are never grounded, and tiny gaps need no snap.
    #[test]
    fn walkable_and_snap_rules() {
        let flat = Ground {
            normal: Vec3::Y,
            gap: 0.01,
        };
        let steep = Ground {
            normal: Vec3::new(0.8, 0.6, 0.0),
            gap: 0.0,
        };
        assert!(walkable(Some(flat), false).is_some());
        assert!(walkable(Some(flat), true).is_none());
        assert!(walkable(Some(steep), false).is_none());
        assert_eq!(snap_velocity(flat.gap, 0.016), 0.0);
        assert!(snap_velocity(0.3, 0.016) < -10.0);
    }
}
