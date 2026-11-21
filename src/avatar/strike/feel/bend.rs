//! The bend of a strike toward its aim: how far the body stoops to a low target, and how the path
//! of the tool turns about the shoulder to pass through the point the crosshair chose.

use bevy::prelude::*;

use super::super::super::pose::{Core, Pose};
use super::super::super::rig::{Seg, globals};
use super::super::super::tune::{PELVIS, SHOULDER, WAIST};
use super::super::key::Key;
use super::tune::{LOW, STOOP, TILT, YAW};

/// How a strike bends toward its aim. The default bends nothing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Bend {
    /// How far the body stoops, 0 upright to 1 bent to a target at its feet.
    pub low: f32,
    /// How much of the stoop the legs carry, 0 while walking to 1 standing.
    pub still: f32,
    /// Turn of the path up about the shoulder, in radians: negative aims down.
    pub tilt: f32,
    /// Turn of the path to the left about the shoulder, in radians.
    pub yaw: f32,
}

/// Returns how far a body stoops to a target `height` metres up in its frame: not at all above the
/// hips, fully at the soles.
pub fn low(height: f32) -> f32 {
    ((LOW.0 - height) / LOW.1).clamp(0.0, 1.0)
}

/// Returns the pose of the trunk of a body stooped by `low`, its legs carrying `still` of the drop
/// and the lean; the chest bends whatever the legs do.
pub fn stoop(low: f32, still: f32) -> Pose {
    Pose::default()
        .with(Core::Lift, -STOOP[0] * low * still)
        .with(Core::Pitch, STOOP[1] * low * still)
        .with(Core::Spine, STOOP[2] * low)
}

/// Returns where the right shoulder of a body standing at rest is, in the frame of the body.
pub fn shoulder() -> Vec3 {
    Vec3::from_array(PELVIS) + Vec3::from_array(WAIST) + Vec3::from_array(SHOULDER)
}

impl Bend {
    /// Returns the bend toward `aim`, a point in the frame of the body, for legs that carry `still`
    /// of a stoop, or none without an aim. The tilt and the yaw are those of the aim seen from the
    /// right shoulder of the torso as it stoops at the moment of the blow.
    pub fn of(aim: Option<Vec3>, still: f32) -> Self {
        let Some(aim) = aim else {
            return Self::default();
        };
        let low = low(aim.y);
        let torso = globals(&stoop(low, still))[Seg::Torso as usize];
        let pivot = torso.transform_point(Vec3::from_array(SHOULDER));
        let d = torso.rotation.inverse() * (aim - pivot);
        let flat = Vec2::new(d.x, d.z).length().max(1e-3);
        Self {
            low,
            still,
            tilt: d.y.atan2(flat).clamp(TILT.0, TILT.1),
            yaw: (-d.x).atan2((-d.z).max(0.05)).clamp(-YAW, YAW),
        }
    }

    /// Returns `key`, a key of the torso frame, turned `share` of the way toward the aim about the
    /// right shoulder, with the chest bent by the stoop.
    pub fn turn(&self, key: &Key, share: f32) -> Key {
        let share = share.clamp(0.0, 1.0);
        let pivot = Vec3::from_array(SHOULDER);
        let turn =
            Quat::from_rotation_y(self.yaw * share) * Quat::from_rotation_x(self.tilt * share);
        Key {
            grip: pivot + turn * (key.grip - pivot),
            axis: (turn * key.axis).normalize_or(key.axis),
            twist: key.twist,
            spine: key.spine + STOOP[2] * self.low * share,
        }
    }
}
