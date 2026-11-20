//! Recoil: how a blow kicks the camera of the robot that struck it, along the blow and by its energy,
//! on top of the shake of the hitstop.

use bevy::prelude::*;

use super::stop::{Hitstop, frames};
use super::tune::{CAMERA_KICK, CAMERA_SHAKE, STOP};

/// Returns how hard a blow of `energy` joules kicks the camera, from 0.5 for the lightest hitstop to 2
/// for the longest: it follows the frames of the hitstop.
pub fn gain(energy: f32) -> f32 {
    0.5 + 1.5 * (frames(energy) - STOP[0]) / (STOP[2] - STOP[0])
}

/// Moves the local `camera`, whose place in the world is `global`, by the shake of `stop` at `share`
/// and its kick along the blow, on top of where the rig put it: a camera the rig left alone since the
/// last frame loses last frame's offset first.
pub fn steady(camera: &mut Transform, global: &GlobalTransform, stop: &mut Hitstop, share: f32) {
    let left = (stop.left / stop.total.max(f32::EPSILON)).max(0.0);
    let toward = camera.rotation * (global.rotation().inverse() * stop.along);
    let jitter = Vec3::new(1.0, -0.5, 0.0) * CAMERA_SHAKE * share;
    let kick = toward * CAMERA_KICK * stop.gain * left;
    let shake = (jitter + kick) * stop.camera;
    let [written, before] = stop.lens;
    let base = match camera.translation == written {
        true => written - before,
        false => camera.translation,
    };
    camera.translation = base + shake;
    stop.lens = [camera.translation, shake];
}
