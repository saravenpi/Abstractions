//! The axes the rig works in. They belong to the target, so they turn with the world.

use bevy::prelude::*;

use super::io::{Kind, Target};
use super::smooth::closed;
use super::tune::{CHASE_ALONG, CHASE_BAND, CHASE_LEVEL};
use crate::render::smoothstep;

/// Returns the axes that stand along `up` and face `forward` as nearly as that allows.
fn upright(up: Vec3, forward: Vec3) -> Quat {
    let up = up.try_normalize().unwrap_or(Vec3::Y);
    let right = forward
        .cross(up)
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector());
    Quat::from_mat3(&Mat3::from_cols(right, up, right.cross(up))).normalize()
}

/// Returns the axes that face `forward` with their roof toward `roof` as nearly as that allows.
fn facing(forward: Vec3, roof: Vec3) -> Quat {
    let back = -forward.try_normalize().unwrap_or(Vec3::NEG_Z);
    let right = roof
        .cross(back)
        .try_normalize()
        .unwrap_or_else(|| back.any_orthonormal_vector());
    Quat::from_mat3(&Mat3::from_cols(right, back.cross(right), back)).normalize()
}

/// Returns the axes of `target`: upright for a body, the hull axes for the ship.
pub(super) fn frame(target: &Target) -> Quat {
    match target.kind {
        Kind::Ship => facing(target.forward, target.up),
        Kind::OnFoot | Kind::Flying => upright(target.up, target.forward),
    }
}

/// Returns the axes the camera of `target` aims to look along, `third` of the way to a third person view.
///
/// For a body they are its own. The chase camera of the ship keeps the axes of the hull in
/// space, where nothing is up. Low in an atmosphere it levels toward the local up, which only
/// rolls it around the nose. When it may `follow`, as with auto-follow on, it also turns at
/// speed from the nose toward where the ship is going; else it keeps to the nose, so the angle
/// the player chose around the hull holds.
pub(super) fn chase(target: &Target, third: f32, follow: bool) -> Quat {
    if target.kind != Kind::Ship {
        return frame(target);
    }
    let near = target.ground.length().min(1.0) * third;
    let speed = target.velocity.length() * f32::from(u8::from(follow));
    let along = target.velocity.try_normalize().unwrap_or(target.forward);
    let fast = smoothstep(CHASE_BAND.0, CHASE_BAND.1, speed);
    let share = near * CHASE_ALONG * fast * along.dot(target.forward).max(0.0);
    let ahead = target.forward.lerp(along, share);
    let level = target.ground.try_normalize().unwrap_or(target.up);
    facing(ahead, target.up.lerp(level, near * CHASE_LEVEL))
}

/// Returns the turn `rel` of the camera away from the axes of a target that turned by `swept`
/// during the frame, after it closed `share` of the way to `goal`.
///
/// The sweep is expressed in the axes of the target and so are the turns: nothing here is
/// measured against the world, which is why a rebase cannot disturb it.
pub(super) fn carried(rel: Quat, swept: Vec3, goal: Quat, share: f32) -> Quat {
    let left = Quat::from_scaled_axis(-swept) * rel;
    left.slerp(goal, share).normalize()
}

/// What one step knows about the target besides its raw data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Stance {
    /// Axes of the target in render space.
    pub frame: Quat,
    /// Velocity of the target in its own axes.
    pub drift: Vec3,
    /// Angular velocity of the target in its own axes.
    pub spin: Vec3,
    /// Frame time in seconds.
    pub dt: f32,
    /// Whether this step follows a cut: nothing is smoothed.
    pub fresh: bool,
}

impl Stance {
    /// Returns the stance of `target` for a step of `dt` seconds.
    pub(super) fn of(target: &Target, dt: f32, fresh: bool) -> Self {
        let frame = frame(target);
        Self {
            frame,
            drift: frame.inverse() * target.velocity,
            spin: frame.inverse() * target.spin,
            dt,
            fresh,
        }
    }

    /// Returns the share of a gap to close this step at `rate`: all of it after a cut.
    pub(super) fn gain(&self, rate: f32) -> f32 {
        if self.fresh {
            return 1.0;
        }
        closed(rate, self.dt)
    }
}
