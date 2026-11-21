//! A body that moves on its own, for the tests and the takes of the footsteps: a script gives
//! its state at each moment, the real leg cycle and the real contacts turn it into steps.

use super::contact::{Footfall, contacts};
use crate::audio::Step;
use crate::avatar::animate::Anim;
use crate::avatar::{AvatarPose, Mode};

/// Frames per second of the game the body is moved at.
const FPS: f32 = 60.0;

/// Returns the state of a body at `speed` metres per second in `mode`, crouched by `crouch`.
pub(crate) fn moving(mode: Mode, speed: f32, crouch: f32) -> AvatarPose {
    AvatarPose {
        mode,
        speed,
        crouch,
        ..AvatarPose::default()
    }
}

/// Moves a body for `seconds` under a `gravity`, its state at each moment given by `script`,
/// and returns every contact it makes with the ground with the second it makes it at. The
/// distance walked grows with the speed while the body walks or runs.
pub(crate) fn live(
    script: impl Fn(f32) -> AvatarPose,
    gravity: f32,
    seconds: f32,
) -> Vec<(f32, Step)> {
    let dt = 1.0 / FPS;
    let (mut anim, mut feet, mut walked) = (Anim::default(), Footfall::default(), 0.0);
    let mut out = Vec::new();
    for frame in 0..(seconds * FPS) as usize {
        let t = frame as f32 * dt;
        let mut pose = script(t);
        if matches!(pose.mode, Mode::Idle | Mode::Walk | Mode::Run) {
            walked += pose.speed.abs() * dt;
        }
        pose.stride_distance = walked;
        anim.input = pose;
        anim.clock.tick(&pose, dt);
        let made = contacts(&mut feet, &anim, gravity, dt);
        out.extend(made.into_iter().flatten().map(|step| (t, step)));
    }
    out
}
