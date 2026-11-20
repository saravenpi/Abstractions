//! Skids, and the continuity of every touchdown: no joint jumps from the air to the ground and up.

use bevy::prelude::*;

use super::flown::JOINT_STEP;
use super::gap;
use super::stage::{SETTLE, Touchdown};
use crate::avatar::Mode;
use crate::avatar::pose::{Core, Pose};
use crate::avatar::rig::{Seg, globals};
use crate::avatar::touch::floor::lowest;
use crate::avatar::tune::SOLE_Y;
use crate::gear::Mode as Flight;
use crate::player::wrap;

/// Largest change of a joint in one frame while the knees give way under a hard landing on the
/// spot and push the body back up, in radians: the bound every landing keeps.
const CROUCH_STEP: f32 = 0.16;

/// Largest move of the pelvis up or down in one frame of a touchdown, in meters, on top of what
/// the whole-body turn of that frame carries it by.
const LIFT_STEP: f32 = 0.06;

/// Most the pelvis rides up or down per radian the whole body turns, in meters: from lying on
/// the back to sitting up, a tumbling body lifts its pelvis by about half a meter in a quarter
/// turn.
const RIDE: f32 = 0.4;

/// Largest whole-body turn of a roll in one frame, in radians: no snap, a full turn takes at
/// least a dozen frames.
const TURN_STEP: f32 = 0.6;

/// Returns a body that came in at `velocity` in the gear mode `flight`, lying as `prone` as the
/// gear says, after `frames` frames on the ground.
fn landed(flight: Flight, velocity: Vec3, prone: f32, frames: usize) -> Touchdown {
    let mode = if flight.winged() {
        Mode::Glide
    } else {
        Mode::Fall
    };
    let mut body = Touchdown::new(flight, velocity, (0.0, prone), mode);
    body.hit();
    body.ground(frames);
    body
}

/// A glide brought in flat skids on its belly, low on the ground; a body that came in upright
/// skids on its feet, sitting back against the way it slides, its lead boot ahead.
#[test]
fn the_skid_leans_the_way_it_came_in() {
    let belly = landed(Flight::Glide, Vec3::new(0.0, -3.0, -28.0), 1.0, 60);
    assert_eq!(
        (belly.state.mode, belly.pose.mode),
        (Flight::Slide, Mode::Skid)
    );
    let pose = belly.anim.pose;
    assert!(pose.core(Core::Pitch) > 1.2, "belly leans {pose:?}");
    assert!(pose.core(Core::Lift) < -0.5, "belly lifts {pose:?}");
    assert!((lowest(&pose) - SOLE_Y).abs() < 0.01, "belly floats");
    let feet = landed(Flight::Fall, Vec3::new(0.0, -3.0, -28.0), 0.0, 60);
    assert_eq!(feet.pose.mode, Mode::Skid);
    let pose = feet.anim.pose;
    let frames = globals(&pose);
    let (boot, hips) = (frames[Seg::BootL as usize], frames[Seg::Pelvis as usize]);
    assert!(pose.core(Core::Pitch) < -0.3, "feet first leans {pose:?}");
    assert!(
        boot.translation.z < hips.translation.z - 0.3,
        "the lead boot is not ahead"
    );
    assert!((lowest(&pose) - SOLE_Y).abs() < 0.01, "feet first floats");
    let right = landed(Flight::Fall, Vec3::new(28.0, -3.0, 0.0), 0.0, 60);
    let roll = right.anim.pose.core(Core::Roll);
    assert!(roll < -0.25, "sliding right leans {roll}");
}

/// Returns the largest change of a joint, of the lift less what the turn carries, and of the
/// whole-body turn in one frame over the frames of `body` once the gear took the touchdown, the
/// turn taken out of the joints. The two frames of the hit itself belong to the fall pose, whose legs stop reaching as the
/// fall speed it reads eases to zero.
fn worst(body: &Touchdown) -> Vec3 {
    let still = |pose: &Pose| pose.with(Core::Lift, 0.0);
    let steps = body.frames[SETTLE + 2..].windows(2).map(|pair| {
        let ((was, turned), (now, turn)) = (pair[0], pair[1]);
        let lift = (now.core(Core::Lift) - was.core(Core::Lift)).abs();
        let spun = Vec2::new(wrap(turn.x - turned.x), wrap(turn.y - turned.y));
        let spun = spun.abs().max_element();
        Vec3::new(gap(&still(&was), &still(&now)), lift - RIDE * spun, spun)
    });
    steps.fold(Vec3::ZERO, Vec3::max)
}

/// From the air onto the ground and back up, in every family, no joint jumps more than
/// [`JOINT_STEP`] in a frame, the pelvis rides smoothly and the turn of a roll never snaps. Only
/// the knees giving way under a hard landing on the spot move faster, inside [`CROUCH_STEP`].
#[test]
fn touchdowns_never_jump_a_joint() {
    let cases = [
        (Flight::Glide, Vec3::new(0.0, -3.0, -28.0), 1.0, JOINT_STEP),
        (Flight::Fall, Vec3::new(0.0, -3.0, -28.0), 0.0, JOINT_STEP),
        (Flight::Fall, Vec3::new(0.0, -14.0, -16.0), 0.0, JOINT_STEP),
        (Flight::Fall, Vec3::new(16.0, -14.0, 0.0), 0.0, JOINT_STEP),
        (Flight::Fall, Vec3::new(0.0, -42.0, -42.0), 0.0, JOINT_STEP),
        (Flight::Glide, Vec3::new(0.0, -16.0, -20.0), 1.0, JOINT_STEP),
        (Flight::Fall, Vec3::new(0.0, -20.0, -1.0), 0.0, CROUCH_STEP),
        (Flight::Off, Vec3::new(0.0, -22.0, 0.0), 0.0, CROUCH_STEP),
    ];
    for (flight, velocity, prone, bound) in cases {
        let body = landed(flight, velocity, prone, 240);
        let step = worst(&body);
        let what = format!("{flight:?} at {velocity}");
        assert!(step.x < bound, "{what}: a joint jumps by {}", step.x);
        assert!(step.y < LIFT_STEP, "{what}: the pelvis jumps by {}", step.y);
        assert!(step.z < TURN_STEP, "{what}: the trunk snaps by {}", step.z);
        assert_eq!(body.anim.clock.turn, Vec2::ZERO, "{what}: still turned");
    }
}
