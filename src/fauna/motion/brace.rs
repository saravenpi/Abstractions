//! The brace: how a body holds itself while it prepares and makes an attack, laid over the walk. The
//! body is lowered on its legs, its weight moved along its heading, pitched and turned about its
//! hips. The feet stay where the gait planted them, so the legs bend to meet the hips. The part that
//! strikes moves by turning its joints, never by sliding: the neck bends back and throws the head out
//! along its reach, the front legs lift and strike, the tail swings about its root.

use bevy::math::{Quat, Vec3};

use super::rest::Place;
use super::rig::Motion;
use crate::life::body::{Body, Leg};

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
    /// Radians the neck is drawn back about its root, positive back and up, negative head down.
    pub neck: f32,
    /// How far the head is thrown out along the reach of the neck, 0 at rest to 1 at full stretch.
    pub reach: f32,
    /// How high the front feet are lifted, as a share of the leg.
    pub lift: f32,
    /// How far the front feet strike ahead of where they stand, as a share of the leg.
    pub claw: f32,
    /// Radians the tail swings to the right about its root.
    pub lash: f32,
    /// How far the jaw opens for a bite, 0 shut to 1 wide.
    pub gape: f32,
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

/// Returns the root every part of `body` in `m` hangs from: the drawn root raised by the call it
/// makes, then held as its brace holds it. The pose and the chains both start from it, so a crouch or
/// a rear carries the neck and the tail with the torso.
pub fn held(m: &Motion, body: &Body) -> Place {
    let song = m.song.as_ref().filter(|s| s.live());
    let raise = song.map_or(0.0, |s| s.raise(body.height));
    m.brace
        .placed((m.root.place + Vec3::Y * raise, m.root.turn), body.height)
}

/// Returns the point front leg `leg` of `m` reaches for, `foot` being where it stands and `reach` its
/// whole length: lifted by the brace's lift and thrown ahead along the heading by its claw. Other
/// legs stand where they are. The leg solver keeps every bone its length.
pub fn pawed(m: &Motion, leg: &Leg, foot: Vec3, reach: f32) -> Vec3 {
    if leg.rank != 0 || (m.brace.lift == 0.0 && m.brace.claw == 0.0) {
        return foot;
    }
    let ahead = (m.root.turn * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
    foot + (Vec3::Y * m.brace.lift + ahead * m.brace.claw) * reach
}
