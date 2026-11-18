//! The puppet: everything of a companion that follows from its pose alone, with no brain.
//!
//! A companion driven from somewhere else, by a pose that arrives each frame, has no brain to
//! tell how it moves. [`track`] measures its velocity from the pose itself, in the frame of
//! the anchor, and [`pose`] bobs, leans and turns the head of every companion from its
//! [`Float`], so the local one and a remote one fly the same pose and lay the same trail.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::motion::{Float, POSE_RATE, blend, lean};
use super::soar::soar;
use super::spring::ramp;
use super::tune::{BOB, GAIN_RATE, MAX_STEP, TAIL_SPEED};
use crate::space::{Anchor, Frame};

/// The parts of one companion its pose moves, on its root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parts {
    /// The rig: the parent of everything visible, which bobs and leans.
    pub rig: Entity,
    /// The head on the rig, turned by the neck.
    pub head: Entity,
}

/// Where a companion was on the last frame, in the frame of the anchor, on its root.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Tracked(pub Option<(Frame, DVec3)>);

/// Returns the velocity in render axes of a body that went from `was` to `now` in the frame of
/// `anchor` in `dt` seconds: zero on the frame the frame itself changes.
pub fn moved(anchor: &Anchor, was: Option<(Frame, DVec3)>, now: DVec3, dt: f32) -> Vec3 {
    match was {
        Some((frame, was)) if frame == anchor.frame => {
            (anchor.rotation * (now - was)).as_vec3() / dt.max(1e-4)
        }
        _ => Vec3::ZERO,
    }
}

/// Measures how every companion without a brain moves, from its pose alone: its velocity, how
/// fast that changes, and how far it is into flight, which is how fast it goes.
pub fn track(
    time: Res<Time>,
    anchor: Res<Anchor>,
    mut bots: Query<(&Transform, &mut Float, &mut Tracked), (With<Companion>, Without<Brain>)>,
) {
    let dt = time.delta_secs().min(MAX_STEP);
    for (at, mut float, mut tracked) in &mut bots {
        let now = anchor.to_frame(at.translation);
        let vel = moved(&anchor, tracked.0, now, dt);
        tracked.0 = Some((anchor.frame, now));
        let gained = (vel - float.vel) / dt.max(1e-4);
        let eased = float.gain + (gained - float.gain) * blend(GAIN_RATE, dt);
        (float.gain, float.vel) = (eased, vel);
        float.flight = ramp(TAIL_SPEED.0, TAIL_SPEED.1, vel.length());
    }
}

/// Returns the lean of a body for its motion: the soft lean of a hover, turned into the flying
/// pose as far as it flies. Seated it only hovers.
pub fn tilt(at: &Transform, float: &Float) -> Quat {
    let own = at.rotation.inverse();
    let (local, gain) = (own * float.vel, own * float.gain);
    let flight = float.flight * f32::from(u8::from(!float.seated));
    lean(local).slerp(soar(local, gain), flight)
}

/// Bobs, leans and poses the rig of every companion on top of where its body floats, and
/// turns its head on its neck.
pub fn pose(
    time: Res<Time>,
    bots: Query<(&Transform, &Float, &Parts), With<Companion>>,
    mut parts: Query<&mut Transform, Without<Companion>>,
) {
    let (now, dt) = (time.elapsed_secs_wrapped(), time.delta_secs().min(MAX_STEP));
    let share = blend(POSE_RATE, dt);
    for (at, float, own) in &bots {
        if let Ok(mut head) = parts.get_mut(own.head) {
            head.rotation = float.neck.rotation();
        }
        let Ok(mut rig) = parts.get_mut(own.rig) else {
            continue;
        };
        let want = tilt(at, float) * float.pose.rotation();
        rig.rotation = rig.rotation.slerp(want, share);
        let lift = float.pose.lift + BOB.0 * (now * BOB.1).sin();
        rig.translation.y += (lift - rig.translation.y) * share;
    }
}
