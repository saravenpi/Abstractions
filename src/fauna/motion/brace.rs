//! The brace: how a body holds itself while it prepares and makes an attack, laid over the walk. The
//! body is lowered on its legs, its weight moved along its heading, pitched and turned about its
//! hips. The feet stay where the gait planted them, so the legs bend to meet the hips.

use bevy::math::{Quat, Vec3};

/// How a body is held for an attack. All zero is a body at ease.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Brace {
    /// How far the hips drop, as a share of the hip height.
    pub drop: f32,
    /// Metres the weight moves back along the heading; negative forward.
    pub shift: f32,
    /// Radians the body pitches about the hips, positive nose up.
    pub pitch: f32,
    /// Radians the body turns about the vertical through the hips.
    pub yaw: f32,
}

impl Brace {
    /// Returns true for a body at ease.
    pub fn at_ease(&self) -> bool {
        *self == Self::default()
    }

    /// Returns the root `(place, turn)` of a body of hip height `height` as this brace holds it: the
    /// hips are the pivot of the pitch and the turn, then lowered by [`Brace::drop`] and moved by
    /// [`Brace::shift`] along the heading.
    pub fn placed(&self, (place, turn): (Vec3, Quat), height: f32) -> (Vec3, Quat) {
        if self.at_ease() {
            return (place, turn);
        }
        let held = Quat::from_rotation_y(self.yaw) * turn * Quat::from_rotation_x(self.pitch);
        let hips = Vec3::Y * height;
        let moved = turn * Vec3::Z * self.shift - Vec3::Y * self.drop * height;
        (place + turn * hips - held * hips + moved, held)
    }
}
