//! Floating: springy following on foot and in flight, the ride in the cockpit, turning toward
//! what it looks at, bob and lean.

use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::neck::Neck;
use super::pose::Pose;
use super::sense::Around;
use super::solid::Solids;
use super::spot::Probe;
use super::spring::{rate, step};
use super::tune::{
    FEED, FEED_FLIGHT, FLIGHT_LEASH, FLIGHT_STIFFNESS, FLOOR, GAIN_RATE, LEASH, LOOK_AHEAD,
    SCALE_RATE, SEAT, SEAT_RATE, SEAT_SCALE, TILT,
};

/// How fast the body takes a new pose, per second.
pub const POSE_RATE: f32 = 9.0;

/// How the companion moves.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Float {
    /// Velocity in the axes its transform is expressed in, in meters per second.
    pub vel: Vec3,
    /// How fast that velocity changes, eased, in meters per second squared. The flying pose
    /// leans along it.
    pub gain: Vec3,
    /// How far it has moved from hovering, 0, to flying, 1.
    pub flight: f32,
    /// How the body is posed on top of floating.
    pub pose: Pose,
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
/// the wings never makes it swing. Only the goal looks ahead for the ground: the body is pulled
/// toward it by the spring and is itself held clear of what lies under it alone, so nothing
/// ever moves it in one step. Truly left behind, by a fast travel, it starts again out of view
/// and flies in: then it returns true.
pub fn follow(
    probe: &Probe,
    brain: &Brain,
    at: &mut Transform,
    float: &mut Float,
    dt: f32,
) -> bool {
    let (goal, clearance) = brain.goal();
    let flight = brain.flight;
    let lead = brain.drift * (FEED + (FEED_FLIGHT - FEED) * flight);
    let target = probe.clear_ahead(goal, lead, (LOOK_AHEAD * flight, clearance));
    let mut offset = at.translation + lead * dt - target;
    let mut slip = float.vel - lead;
    let lost = offset.length() > LEASH + (FLIGHT_LEASH - LEASH) * flight;
    if lost {
        (offset, slip) = (brain.entry() - target, Vec3::ZERO);
        float.vel = lead;
    }
    let stiff = rate(offset, slip).max(FLIGHT_STIFFNESS * flight);
    let (offset, slip) = step(offset, slip, stiff, dt);
    at.translation = probe.clear(target + offset, FLOOR);
    let gained = (slip + lead - float.vel) / dt.max(1e-4);
    float.gain += (gained - float.gain) * blend(GAIN_RATE, dt);
    float.vel = slip + lead;
    let up = probe.up(at.translation);
    at.rotation = float
        .neck
        .turn(at.rotation, brain.look - at.translation, up, dt);
    grow(at, 1.0, dt);
    lost
}

/// Follows the player as [`follow`] does, each step swept among the drawn solids so that a
/// wall, a roof or the ship's hull stops it and it slides along them and round them. A start again out of
/// view after a real loss is not swept: nothing sees it.
pub fn chase(
    probe: &Probe,
    solids: &mut Solids,
    brain: &Brain,
    (at, float): (&mut Transform, &mut Float),
    dt: f32,
) {
    let from = at.translation;
    if !follow(probe, brain, at, float, dt) {
        solids.sweep((from, brain.stance.eye), at, float, dt);
    }
}

/// Rides in the cockpit: a spring toward the seat in the space of the pilot seat, shrunk to its
/// cockpit size. Its head turns to `look`, a point in that space, and its body follows the head
/// as far as it stays upright, so the screen faces what it looks at from under the pilot too.
pub fn ride(look: Vec3, at: &mut Transform, float: &mut Float, dt: f32) {
    let seat = Vec3::from_array(SEAT);
    let (offset, vel) = step(at.translation - seat, float.vel, SEAT_RATE, dt);
    at.translation = seat + offset;
    float.vel = vel;
    let toward = look - at.translation;
    at.rotation = float.neck.turn(at.rotation, toward, Vec3::Y, dt);
    grow(at, SEAT_SCALE, dt);
}

/// Moves every companion that has a brain: following on foot or riding in the ship. What its
/// motion shows, the bob, the lean and the pose, is left to `puppet::pose`.
pub fn float(
    around: Around,
    mut solids: Solids,
    mut bots: Query<(&mut Transform, &mut Float, &Brain), With<Companion>>,
) {
    let dt = around.dt();
    for (mut at, mut float, brain) in &mut bots {
        (float.flight, float.pose) = (brain.flight, brain.pose);
        if float.seated {
            let look = around
                .body
                .compute_affine()
                .inverse()
                .transform_point3(brain.look);
            ride(look, &mut at, &mut float, dt);
        } else {
            chase(
                &around.probe(),
                &mut solids,
                brain,
                (&mut at, &mut float),
                dt,
            );
        }
    }
}
