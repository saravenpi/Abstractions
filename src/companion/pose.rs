//! Body language: how each idle moves the body and the hands, as pure curves of time.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::idle::Idle;
use super::spring::ramp;

/// Seconds an idle takes to ease in and out, so every idle starts and ends at rest.
const EASE: f32 = 0.6;

/// How far the dozing body sinks, in meters.
const DOZE_SINK: f32 = 0.22;

/// An offset of the body from its plain hover, added on top of following and bobbing.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose {
    /// Extra height in meters.
    pub lift: f32,
    /// Turn about the vertical in radians.
    pub yaw: f32,
    /// Nod in radians, positive looks up.
    pub pitch: f32,
    /// Head tilt to the side in radians.
    pub roll: f32,
    /// How far the hands are stretched out, from 0 to 1.
    pub spread: f32,
    /// How much the hands wave in turn, from 0 to 1.
    pub wave: f32,
}

impl Pose {
    /// Returns the rotation of the pose: yaw, then nod, then tilt.
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw)
            * Quat::from_rotation_x(self.pitch)
            * Quat::from_rotation_z(self.roll)
    }
}

/// Returns the weight of an idle at `t` seconds of `len`: 0 at both ends, 1 in between.
pub fn envelope(t: f32, len: f32) -> f32 {
    ramp(0.0, EASE, t) * ramp(0.0, EASE, len - t)
}

/// Returns the pose of the idles that move the whole body, weighted by `e`, or nothing for the quiet ones.
fn lively(idle: Idle, t: f32, len: f32, e: f32) -> Option<Pose> {
    let rest = Pose::default();
    match idle {
        Idle::Spin => Some(Pose {
            yaw: TAU * ramp(0.0, len, t),
            lift: 0.06 * e,
            spread: 0.5 * e,
            ..rest
        }),
        Idle::Dance => Some(Pose {
            lift: 0.07 * e * (t * 8.0).sin().abs(),
            yaw: 0.5 * e * (t * 4.0).sin(),
            roll: 0.24 * e * (t * 8.0).sin(),
            spread: 0.4 * e,
            wave: e,
            ..rest
        }),
        Idle::Stretch => Some(Pose {
            lift: 0.05 * e,
            pitch: 0.25 * e,
            spread: e,
            ..rest
        }),
        _ => None,
    }
}

/// Returns the pose of `idle` at `t` seconds of its `len` seconds.
pub fn idle_pose(idle: Idle, t: f32, len: f32) -> Pose {
    let e = envelope(t, len);
    let rest = Pose::default();
    let quiet = match idle {
        Idle::LookAround => Pose {
            yaw: 0.95 * e * (t * 1.3).sin(),
            pitch: 0.12 * e * (t * 0.9).sin(),
            ..rest
        },
        Idle::Hum => Pose {
            lift: 0.045 * e * (t * 6.0).sin(),
            roll: 0.14 * e * (t * 3.0).sin(),
            ..rest
        },
        Idle::SkyPeek => Pose {
            pitch: 0.2 * e,
            lift: 0.08 * e,
            ..rest
        },
        _ => Pose {
            roll: 0.16 * e * (t * 1.1).sin(),
            ..rest
        },
    };
    lively(idle, t, len, e).unwrap_or(quiet)
}

/// Returns the pose after `t` seconds of dozing: sunk, tilted and bobbing slowly.
pub fn doze_pose(t: f32) -> Pose {
    let e = ramp(0.0, 3.0, t);
    Pose {
        lift: -DOZE_SINK * e + 0.015 * e * (t * 0.9).sin(),
        pitch: -0.22 * e,
        roll: 0.12 * e,
        ..Pose::default()
    }
}
