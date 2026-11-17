//! Floating: springy following on foot, the ride in the cockpit, turning toward what it looks at, bob and lean.

use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::pose::Pose;
use super::sense::Around;
use super::spot::Probe;
use super::spring::{rate, step};
use super::tune::{BOB, CLEARANCE, FEED, FLOOR, LEASH, SEAT, SEAT_RATE, TILT, TURN_RATE};

/// How fast the body takes a new pose, per second.
const POSE_RATE: f32 = 9.0;

/// How the companion moves.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Float {
    /// Velocity in the axes its transform is expressed in, in meters per second.
    pub vel: Vec3,
    /// True while it rides in the cockpit as a child of the camera.
    pub seated: bool,
}

/// Marks the part of the companion that bobs, leans and poses: the parent of everything visible.
#[derive(Component, Debug, Clone, Copy)]
pub struct Rig;

/// Returns the share of the way to a target covered in `dt` seconds at `rate` per second.
pub fn blend(rate: f32, dt: f32) -> f32 {
    1.0 - (-rate * dt).exp()
}

/// Returns the rotation that points the screen, which faces -Z, along `dir` with `up` upward.
pub fn facing(dir: Vec3, up: Vec3) -> Quat {
    Transform::IDENTITY.looking_to(dir, up).rotation
}

/// Returns the lean of a body moving at `local` meters per second in its own axes: into its direction of travel.
pub fn lean(local: Vec3) -> Quat {
    let tilt = (Vec2::new(local.x, local.z) * TILT.0).clamp_length_max(TILT.1);
    Quat::from_rotation_x(tilt.y) * Quat::from_rotation_z(-tilt.x)
}

/// Follows the player on foot: a spring toward the goal, carried partly by the player's own velocity.
///
/// The goal and the body both stay clear of the ground and the water. Left very far behind, it
/// reappears behind the player at rest.
pub fn follow(probe: &Probe, brain: &Brain, at: &mut Transform, float: &mut Float, dt: f32) {
    let (goal, clearance) = brain.goal();
    let target = probe.clear(goal, clearance);
    let lead = brain.drift * FEED;
    let offset = at.translation + lead * dt - target;
    if offset.length() > LEASH {
        at.translation = probe.clear(brain.stance.behind(), CLEARANCE);
        float.vel = Vec3::ZERO;
    } else {
        let slip = float.vel - lead;
        let (offset, vel) = step(offset, slip, rate(offset, slip), dt);
        at.translation = probe.clear(target + offset, FLOOR);
        float.vel = vel + lead;
    }
    let want = facing(brain.look - at.translation, probe.up(at.translation));
    at.rotation = at.rotation.slerp(want, blend(TURN_RATE, dt));
}

/// Rides in the cockpit: a spring toward the seat in camera space, looking at `look` in that space.
pub fn ride(look: Vec3, at: &mut Transform, float: &mut Float, dt: f32) {
    let seat = Vec3::from_array(SEAT);
    let (offset, vel) = step(at.translation - seat, float.vel, SEAT_RATE, dt);
    at.translation = seat + offset;
    float.vel = vel;
    let want = facing(look - at.translation, Vec3::Y);
    at.rotation = at.rotation.slerp(want, blend(TURN_RATE, dt));
}

/// Bobs, leans and poses the rig on top of where the body floats.
fn sway(rig: &mut Transform, pose: &Pose, local: Vec3, (now, dt): (f32, f32)) {
    let share = blend(POSE_RATE, dt);
    rig.rotation = rig.rotation.slerp(lean(local) * pose.rotation(), share);
    let lift = pose.lift + BOB.0 * (now * BOB.1).sin();
    rig.translation.y += (lift - rig.translation.y) * share;
}

/// Moves the companion: following on foot or riding in the ship, then the bob, the lean and the pose.
pub fn float(
    around: Around,
    bot: Single<(&mut Transform, &mut Float, &Brain), With<Companion>>,
    mut rig: Single<&mut Transform, (With<Rig>, Without<Companion>)>,
) {
    let (mut at, mut float, brain) = bot.into_inner();
    let dt = around.dt();
    if float.seated {
        let look = around
            .camera
            .affine()
            .inverse()
            .transform_point3(brain.look);
        ride(look, &mut at, &mut float, dt);
    } else {
        follow(&around.probe(), brain, &mut at, &mut float, dt);
    }
    let local = at.rotation.inverse() * float.vel;
    sway(&mut rig, &brain.pose, local, (around.now(), dt));
}
