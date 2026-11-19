//! The flying pose: the body tips the way one thruster under it must push to move as it does.
//!
//! A body held up by a thruster under it can only push along its own vertical. To gain speed,
//! to turn, or to hold its speed against the air, it has to tip that vertical. So the lean
//! drawn is the lean its real motion needs: nose down at speed, banked into a turn, tipped
//! back when it brakes. Nothing here plays by itself.

use bevy::prelude::*;

use super::tune::{SOAR_DRAG, SOAR_LIMIT};

/// Pull the thruster holds the body up against, in metres per second squared.
const WEIGHT: f32 = 9.81;

/// Least share of the push that stays upward, in metres per second squared, so a body thrown
/// down faster than it falls never asks to hang upside down.
const UPWARD: f32 = 1.0;

/// Returns the tilt of a body that flies at `local` metres per second and gains `gain` metres
/// per second squared, both in its own axes: +Y up, -Z ahead.
///
/// The push it needs is its weight, what it gains and the drag of the air on its speed. The
/// body tips its vertical along that push, by at most [`SOAR_LIMIT`].
pub fn soar(local: Vec3, gain: Vec3) -> Quat {
    let push = Vec3::Y * WEIGHT + gain + local * SOAR_DRAG;
    let flat = Vec2::new(push.x, push.z);
    let Some(way) = flat.try_normalize() else {
        return Quat::IDENTITY;
    };
    let angle = flat.length().atan2(push.y.max(UPWARD)).min(SOAR_LIMIT);
    Quat::from_axis_angle(Vec3::new(way.y, 0.0, -way.x), angle)
}
