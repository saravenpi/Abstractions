//! Floating: springy following on foot, the ride in the cockpit, turning toward what it looks at, bob and lean.

use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::neck::Neck;
use super::pose::Pose;
use super::sense::Around;
use super::spot::Probe;
use super::spring::{rate, step};
use super::tune::{
    BOB, CLEARANCE, FEED, FEED_FLIGHT, FLIGHT_LEASH, FLIGHT_STIFFNESS, FLOOR, LEASH, LOOK_AHEAD,
    SCALE_RATE, SEAT, SEAT_RATE, SEAT_SCALE, TILT,
};
use crate::player::Player;

/// How fast the body takes a new pose, per second.
const POSE_RATE: f32 = 9.0;

/// How the companion moves.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Float {
    /// Velocity in the axes its transform is expressed in, in meters per second.
    pub vel: Vec3,
    /// True while it rides in the cockpit as a child of the player body in the pilot seat.
    pub seated: bool,
    /// How the head is turned from the body, which follows it.
    pub neck: Neck,
}

/// Marks the part of the companion that bobs, leans and poses: the parent of everything visible.
#[derive(Component, Debug, Clone, Copy)]
pub struct Rig;

/// Marks the head on the rig: the shell with its screen and antenna, turned by the neck.
#[derive(Component, Debug, Clone, Copy)]
pub struct Head;

/// Returns the share of the way to a target covered in `dt` seconds at `rate` per second.
pub fn blend(rate: f32, dt: f32) -> f32 {
    1.0 - (-rate * dt).exp()
}

/// Returns the lean of a body moving at `local` meters per second in its own axes: into its direction of travel.
pub fn lean(local: Vec3) -> Quat {
    let tilt = (Vec2::new(local.x, local.z) * TILT.0).clamp_length_max(TILT.1);
    Quat::from_rotation_x(tilt.y) * Quat::from_rotation_z(-tilt.x)
}

/// Eases the size of the body toward `size` times its own.
fn grow(at: &mut Transform, size: f32, dt: f32) {
    let want = Vec3::splat(size);
    if at.scale != want {
        at.scale = at.scale.lerp(want, blend(SCALE_RATE, dt));
        if at.scale.distance(want) < 1e-3 {
            at.scale = want;
        }
    }
}

/// Follows the player: a spring toward the goal, carried partly by the player's own velocity on
/// foot and wholly in flight, so that it keeps its place in the formation at any speed.
///
/// The spring is critically damped in the frame of the player, so an abrupt stop or a fold of
/// the wings never makes it swing. The goal and the body both stay clear of the ground and the
/// water, at speed where they will be as well as where they are. Left very far behind, it
/// reappears behind the player at rest.
pub fn follow(probe: &Probe, brain: &Brain, at: &mut Transform, float: &mut Float, dt: f32) {
    let (goal, clearance) = brain.goal();
    let flight = brain.flight;
    let lead = brain.drift * (FEED + (FEED_FLIGHT - FEED) * flight);
    let target = probe.clear_ahead(goal, lead, (LOOK_AHEAD * flight, clearance));
    let offset = at.translation + lead * dt - target;
    if offset.length() > LEASH + (FLIGHT_LEASH - LEASH) * flight {
        at.translation = probe.clear(brain.stance.behind(), CLEARANCE);
        float.vel = Vec3::ZERO;
    } else {
        let slip = float.vel - lead;
        let stiff = rate(offset, slip).max(FLIGHT_STIFFNESS * flight);
        let (offset, vel) = step(offset, slip, stiff, dt);
        at.translation =
            probe.clear_ahead(target + offset, vel + lead, (LOOK_AHEAD * flight, FLOOR));
        float.vel = vel + lead;
    }
    let up = probe.up(at.translation);
    at.rotation = float
        .neck
        .turn(at.rotation, brain.look - at.translation, up, dt);
    grow(at, 1.0, dt);
}

/// Rides in the cockpit: a spring toward the seat in the space of the pilot seat, looking at
/// `look` in that space, shrunk to its cockpit size.
pub fn ride(look: Vec3, at: &mut Transform, float: &mut Float, dt: f32) {
    let seat = Vec3::from_array(SEAT);
    let (offset, vel) = step(at.translation - seat, float.vel, SEAT_RATE, dt);
    at.translation = seat + offset;
    float.vel = vel;
    let toward = look - at.translation;
    at.rotation = float.neck.turn(at.rotation, toward, Vec3::Y, dt);
    grow(at, SEAT_SCALE, dt);
}

/// Bobs, leans and poses the rig on top of where the body floats.
fn sway(rig: &mut Transform, pose: &Pose, local: Vec3, (now, dt): (f32, f32)) {
    let share = blend(POSE_RATE, dt);
    rig.rotation = rig.rotation.slerp(lean(local) * pose.rotation(), share);
    let lift = pose.lift + BOB.0 * (now * BOB.1).sin();
    rig.translation.y += (lift - rig.translation.y) * share;
}

/// Filter of the rig, which is neither the companion root nor the player body.
type RigOnly = (With<Rig>, Without<Companion>, Without<Player>);

/// Filter of the head, which is neither the rig, the companion root nor the player body.
type HeadOnly = (
    With<Head>,
    Without<Rig>,
    Without<Companion>,
    Without<Player>,
);

/// Moves the companion: following on foot or riding in the ship, the head turned ahead of the
/// body, then the bob, the lean and the pose.
pub fn float(
    around: Around,
    bot: Single<(&mut Transform, &mut Float, &Brain), With<Companion>>,
    (mut rig, mut head): (
        Single<&mut Transform, RigOnly>,
        Single<&mut Transform, HeadOnly>,
    ),
) {
    let (mut at, mut float, brain) = bot.into_inner();
    let dt = around.dt();
    if float.seated {
        let look = around
            .body
            .compute_affine()
            .inverse()
            .transform_point3(brain.look);
        ride(look, &mut at, &mut float, dt);
    } else {
        follow(&around.probe(), brain, &mut at, &mut float, dt);
    }
    head.rotation = float.neck.rotation();
    let local = at.rotation.inverse() * float.vel;
    sway(&mut rig, &brain.pose, local, (around.now(), dt));
}
