//! The roll of the camera around where it looks.
//!
//! Flying, the camera rolls its roof toward the back of the body, a full turn round. It carries
//! its own roof through every turn of the view, so the view passing straight up over the top of a
//! loop, where its heading turns a half turn at once, does not swing the camera. Only the roll the
//! body itself makes is eased.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::io::Target;

/// The roll of the camera and the view it was measured against.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Roll {
    /// Roll of the camera, positive to the right.
    pub angle: f32,
    /// The orientation of the view on the last step while it followed the back of a body.
    look: Option<Quat>,
}

impl Roll {
    /// Eases the roll toward the body by `gain` of the way, for a view with orientation `look`
    /// that rolls by `share` of the body: toward the back of a flying body, else its bank.
    pub(super) fn step(&mut self, target: &Target, look: Quat, (share, gain): (f32, f32)) {
        let back = (target.back != Vec3::ZERO).then_some(target.back);
        let roofed = |last: Quat| last * Quat::from_rotation_z(-self.angle) * Vec3::Y;
        let carried = self
            .look
            .zip(back)
            .and_then(|(last, _)| roof(look, roofed(last)))
            .unwrap_or(self.angle);
        self.look = back.map(|_| look);
        let wanted = back.map_or(Some(target.bank), |back| roof(look, back));
        let wanted = wanted.map_or(carried, |roll| roll * share);
        self.angle = half_turn(carried + half_turn(wanted - carried) * gain);
    }
}

/// Returns `angle` in radians brought within a half turn of zero, so the camera rolls the short
/// way and on round through a full roll without unwinding.
fn half_turn(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Returns the roll that turns the roof of a camera with orientation `look` toward `up`, seen
/// along where it looks, or nothing when the camera looks nearly along it and that roll is
/// meaningless.
fn roof(look: Quat, up: Vec3) -> Option<f32> {
    let seen = look.inverse() * up;
    (seen.truncate().length() > 0.2).then(|| seen.x.atan2(seen.y))
}
