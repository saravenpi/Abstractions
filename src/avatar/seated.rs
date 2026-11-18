//! The pilot in the seat of the ship: the pose that sits the body in the cockpit, its hands on
//! the real stick and throttle, and where the body is put so that its neck is the pivot the
//! first person view turns on.
//!
//! The pose is plain joint angles like every other. The arms are solved for where the grips
//! are, so they may reach across the chest, farther than the limits of the blended poses.

mod sit;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_app;

use std::f32::consts::PI;

use bevy::prelude::*;

use super::pose::{Core, Limb, Pose};
use super::rig::{Seg, globals};
use super::tune::{FOREARM, UPPER_ARM};
use crate::ship::SEAT;
use crate::ship::cockpit::Levers;
use crate::view::FACE;

/// Forward bend of the torso over the pelvis in the seat, toward the controls.
const SPINE: f32 = 0.12;

/// Swing of the thighs, bend of the knees and pitch of the boots: thighs nearly level on the
/// pan, shins down and ahead, soles flat on the floor of the tub.
const LEGS: (f32, f32, f32) = (1.45, 0.945, 0.505);

/// From the wrist to the middle of the palm, which closes on a grip.
const PALM: f32 = 0.045;

/// Returns the pose of the trunk and the legs in the seat, arms hanging.
pub fn rest() -> Pose {
    Pose::default()
        .with(Core::Spine, SPINE)
        .both(Limb::Hip, LEGS.0)
        .both(Limb::Knee, LEGS.1)
        .both(Limb::Ankle, LEGS.2)
}

/// Returns where the root of the body sits in the frame of the player entity, which the ship
/// keeps at the eyes of the pilot at rest: the neck of the seated body is then the pivot the
/// first person view turns the eyes on.
pub fn offset() -> Vec3 {
    -FACE - globals(&rest())[Seg::Head as usize].translation
}

/// Returns where the hands hold the controls `levers`, in the frame of the root of the body:
/// the left hand on the throttle, the right hand on the stick.
pub fn held(levers: &Levers) -> [Vec3; 2] {
    levers.grips().map(|grip| grip - SEAT.as_vec3() - offset())
}

/// Returns the swing of the shoulder, the lift of the arm to the side and the bend of the
/// elbow that put the palm of `side` of a body in `pose` on `grip`, in the frame of its root.
/// A grip out of reach is pointed at with a straight arm.
pub fn reach(pose: &Pose, side: usize, grip: Vec3) -> [f32; 3] {
    let frames = globals(pose);
    let torso = frames[Seg::Torso as usize].rotation;
    let shoulder = frames[Seg::UpperArmL as usize + side].translation;
    let to = torso.inverse() * (grip - shoulder);
    let out = ([-1.0, 1.0][side] * to.x).atan2(-to.y);
    let (down, ahead) = (to.truncate().length(), -to.z);
    let (upper, fore) = (UPPER_ARM, FOREARM + PALM);
    let far = Vec2::new(down, ahead).length();
    let far = far.clamp((upper - fore).abs() + 1.0e-4, upper + fore - 1.0e-4);
    let cosine = (far * far - upper * upper - fore * fore) / (2.0 * upper * fore);
    let elbow = cosine.clamp(-1.0, 1.0).acos().min(PI);
    let lag = (fore * elbow.sin()).atan2(upper + fore * elbow.cos());
    [ahead.atan2(down) - lag, out, elbow]
}

/// Returns the pose in the seat with the hands on `grips`, in the frame of the root of the
/// body: the left hand on the first, the right hand on the second.
pub fn seated(grips: [Vec3; 2]) -> Pose {
    let mut pose = rest();
    for (side, grip) in grips.into_iter().enumerate() {
        let [shoulder, out, elbow] = reach(&rest(), side, grip);
        pose.sides[side][Limb::Shoulder as usize] = shoulder;
        pose.sides[side][Limb::ArmOut as usize] = out;
        pose.sides[side][Limb::Elbow as usize] = elbow;
    }
    pose
}

/// Registers the systems that seat the body: what is drawn of it before the animation, the
/// seated pose over the animated one after.
pub(super) fn systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            sit::shown
                .after(super::AvatarSet::Fill)
                .after(super::dress::wear)
                .before(super::AvatarSet::Animate),
            sit::sit
                .in_set(super::AvatarSet::Animate)
                .after(super::animate::animate)
                .before(super::show::show),
        ),
    );
}
