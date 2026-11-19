//! The records of the robot, the ship and the companion of this machine: each one read from
//! the game, put in the wire frame it belongs in, and said to be at rest or not.

use abstractions_net::wire::{self, CompanionBody, RobotBody, ShipBody};
use abstractions_net::wire::{WARP_ACTIVE, WARP_OFF, WARP_SPOOLING};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::super::bodies::Bodies;
use super::super::frame::{Place, Pose};
use super::super::rate::REST_SPEED;
use super::super::robot::{body, heading};
use crate::avatar::{AvatarPose, Mode};
use crate::companion::Pose as CompanionPose;
use crate::player::Motion;
use crate::ship::{Ship, WarpState};
use crate::space::{Anchor, Placed, up};
use crate::universe::StarId;

/// Where this machine stands when its records are taken.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Here {
    /// The render anchor.
    pub anchor: Anchor,
    /// The frame of the anchor.
    pub place: Place,
    /// Universe time of the frame, the time of every record.
    pub time: f64,
    /// Whether the player sits in the ship.
    pub seated: bool,
    /// Whether the ship of this machine rests on the ground.
    pub landed: bool,
    /// Whether the player stays still with the ground: on foot, or in a ship at rest.
    pub still: bool,
    /// The star this machine was last at, for the frame between the stars.
    pub last: Option<StarId>,
}

/// What one record would carry this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sent<B> {
    /// The wire frame of the body.
    pub frame: wire::Frame,
    /// The body.
    pub body: B,
    /// Whether the thing is at rest.
    pub rest: bool,
}

/// Returns `pose`, given in the anchor frame, in the wire frame it belongs in, with that frame.
/// It goes on the surface when it is `fixed` to it.
fn wired(here: &Here, pose: Pose, fixed: bool, bodies: &mut Bodies) -> Option<(wire::Frame, Pose)> {
    let frame = bodies.choose((here.place, pose.pos), fixed, here.time, here.last)?;
    let pose = bodies.convert(pose, here.place, frame.into(), here.time)?;
    Some((frame, pose))
}

/// Returns the pose of a render transform `at` in the anchor frame, moving at the render
/// velocity `vel`.
fn framed(anchor: &Anchor, at: &GlobalTransform, vel: Vec3) -> Pose {
    let undo = anchor.rotation.inverse();
    Pose {
        pos: anchor.to_frame(at.translation()),
        vel: undo * vel.as_dvec3(),
        rot: (undo * at.rotation().as_dquat()).normalize(),
    }
}

/// Returns the record of the robot whose body is at `at`, moved by `motion` and posed by
/// `pose`. It is at rest standing idle or seated, where the ship carries it.
pub fn robot(
    here: &Here,
    at: &GlobalTransform,
    motion: &Motion,
    pose: &AvatarPose,
    bodies: &mut Bodies,
) -> Option<Sent<RobotBody>> {
    let tilt = DQuat::from_rotation_arc(DVec3::Y, up(&here.anchor, at.translation()).as_dvec3());
    let vel = tilt.as_quat() * motion.velocity;
    let fixed = !here.seated || here.landed;
    let (frame, wire) = wired(here, framed(&here.anchor, at, vel), fixed, bodies)?;
    let idle = pose.mode == Mode::Idle && wire.vel.length() < REST_SPEED;
    Some(Sent {
        frame,
        body: body(pose, wire.pos, wire.vel, heading(wire.pos, wire.rot)),
        rest: idle || pose.mode == Mode::Seated,
    })
}

/// Returns the state of the warp drive as a record holds it, and how far it is.
fn warp(state: WarpState) -> (u8, f32) {
    match state {
        WarpState::Off => (WARP_OFF, 0.0),
        WarpState::Spooling(share) => (WARP_SPOOLING, share as f32),
        WarpState::Active(_) => (WARP_ACTIVE, 1.0),
    }
}

/// Returns the record of the ship of this machine. Its drawn pose is the one at universe
/// time, while the anchor is in its frame; its state otherwise.
pub fn ship(
    here: &Here,
    ship: &Ship,
    placed: Option<&Placed>,
    drive: WarpState,
    bodies: &mut Bodies,
) -> Option<Sent<ShipBody>> {
    let shown = placed.filter(|_| here.anchor.frame == ship.frame);
    let pose = Pose {
        pos: shown.map_or(ship.pos, |placed| placed.pos),
        vel: ship.vel,
        rot: ship.rot,
    };
    let aboard = Here {
        place: Place::from(ship.frame),
        ..*here
    };
    let (frame, wire) = wired(&aboard, pose, ship.landed, bodies)?;
    let (warp, warp_level) = warp(drive);
    let body = ShipBody {
        position: wire.pos,
        velocity: wire.vel.as_vec3(),
        rotation: wire.rot.as_quat(),
        throttle: ship.throttle as f32,
        gear: ship.gear as f32,
        damage: ship.damage as f32,
        landed: ship.landed,
        warp,
        warp_level,
        ..ShipBody::default()
    };
    let rest = ship.landed || wire.vel.length() < REST_SPEED;
    Some(Sent { frame, body, rest })
}

/// Where a thing was on an earlier frame: its wire frame, position and universe time.
pub type Trail = Option<(wire::Frame, DVec3, f64)>;

/// Returns the record of the companion whose root is at `at`, posed as `pose`, which was at
/// `was` on the frame before. It is at rest when it hovers still and the player does not travel.
pub fn companion(
    here: &Here,
    at: &GlobalTransform,
    pose: &CompanionPose,
    was: Trail,
    bodies: &mut Bodies,
) -> Option<Sent<CompanionBody>> {
    let fixed = !here.seated || here.landed;
    let (frame, wire) = wired(here, framed(&here.anchor, at, Vec3::ZERO), fixed, bodies)?;
    let body = CompanionBody {
        position: wire.pos,
        rotation: wire.rot.as_quat(),
        lift: pose.lift,
        yaw: pose.yaw,
        pitch: pose.pitch,
        roll: pose.roll,
        spread: pose.spread,
        wave: pose.wave,
        ..CompanionBody::default()
    };
    let speed = was
        .filter(|(before, ..)| *before == frame)
        .map(|(_, pos, time)| wire.pos.distance(pos) / (here.time - time).max(1e-3));
    let rest = here.still && speed.is_some_and(|speed| speed < REST_SPEED);
    Some(Sent { frame, body, rest })
}
