//! Steering math of the autopilot: the stick that turns the nose onto a course, and the way
//! around a body that stands in it.

use bevy::math::{DQuat, DVec3};

use super::wells::Well;

/// Stick deflection per radian between the nose and the course.
const AIM_GAIN: f64 = 2.0;

/// Lock radii a course keeps from the centre of a body that is not the destination.
const BERTH: f64 = 1.5;

/// Returns the stick of pitch, yaw and roll that turns the nose of a ship with orientation `rot`
/// toward the unit direction `course`. With the flight assist the stick is a turn rate, so the
/// nose settles on the course.
pub fn aim(rot: DQuat, course: DVec3) -> DVec3 {
    let seen = rot.inverse() * course;
    let axis = DVec3::NEG_Z.cross(seen);
    let angle = axis.length().atan2(-seen.z);
    let turn = axis.try_normalize().unwrap_or(DVec3::X);
    turn * (angle * AIM_GAIN).min(1.0)
}

/// Returns the unit direction to fly from `pos` to reach a point `reach` metres away along the
/// unit direction `course`: the course itself, or a direction that passes beside a body whose
/// lock sphere the course would cross.
pub fn detour(wells: &[Well], pos: DVec3, course: DVec3, reach: f64) -> DVec3 {
    let blocks = |well: &&Well| {
        let to = well.pos - pos;
        let along = to.dot(course);
        along > 0.0 && along < reach && (to - course * along).length() < well.lock * BERTH
    };
    let Some(well) = wells.iter().find(blocks) else {
        return course;
    };
    let to = well.pos - pos;
    let miss = course * to.dot(course) - to;
    let side = miss
        .try_normalize()
        .unwrap_or_else(|| course.any_orthonormal_vector());
    (to + side * (well.lock * BERTH * 1.2)).normalize()
}
