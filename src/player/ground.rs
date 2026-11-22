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
    let normal = if normal.y < MIN_GROUND_NORMAL || hit.distance <= 1e-4 {
        let reach = BODY_LENGTH * 0.5 + BODY_RADIUS + SNAP_DISTANCE;
        query
            .cast_ray(transform.translation, down, reach, true, filter)
            .map(|ray| tilt.inverse() * ray.normal)
            .filter(|n| n.y >= MIN_GROUND_NORMAL)
            .unwrap_or(normal)
    } else {
        normal
    };
    Some(Ground {
        normal,
        gap: hit.distance,
    })
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

/// Stops an on-foot collision with an upward-facing slope from converting horizontal speed into
/// extra upward climb: steep unwalkable slopes act as walls when pushed into, and walkable slopes
/// met in the air never lift the body faster than its vertical speed.
pub(super) fn guard_slide(hit: &mut MoveAndSlideHitData, up: Vec3, grounded: bool) {
    let n = *hit.normal;
    let n_up = n.dot(up);
    if n_up <= 0.0 {
        return;
    }
    let v = *hit.velocity;
    let into = v.dot(*n);
    let v_up = v.dot(up);
    let cap = v_up.max(0.0);
    let slide_up = v_up - n_up * into;
    if into >= 0.0 || slide_up <= cap {
        return;
    }
    let n_h = n.reject_from_normalized(up);
    if n_up < MIN_GROUND_NORMAL {
        if let Ok(wall) = Dir3::new(n_h) {
            *hit.normal = wall;
        }
        return;
    }
    let len_sq = n_h.length_squared();
    if !grounded && len_sq > 1e-6 {
        *hit.velocity = v + n_h * ((slide_up - cap) / (n_up * len_sq));
    }
}

/// Returns `at` pulled down to [`SKIN`] above walkable ground when a downward cast finds it
/// within snap range, or `None` when the body is already on it or over a drop.
pub(super) fn snap(
    query: &SpatialQuery,
    (collider, at, rot): (&Collider, Vec3, Quat),
    tilt: Quat,
    filter: &SpatialQueryFilter,
) -> Option<Vec3> {
    let pose = Transform::from_translation(at).with_rotation(rot);
    let ground = probe(query, collider, &pose, tilt, filter)
        .filter(|g| g.normal.y >= MIN_GROUND_NORMAL && g.gap > SKIN)?;
    let down = (tilt * Vec3::NEG_Y).normalize_or(Vec3::NEG_Y);
    Some(at + down * (ground.gap - SKIN))
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
