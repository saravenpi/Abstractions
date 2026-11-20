//! Falls: folded from a glide the body stays prone and blends into the skydiver arch, a fast
//! fall from a jump arches too, a short hop stays upright, and near the ground the body turns
//! upright and reaches for it.

use bevy::prelude::*;

use super::flown::JOINT_STEP;
use super::{FRAME, gap};
use crate::avatar::animate::Anim;
use crate::avatar::blend::Weights;
use crate::avatar::dress::dress;
use crate::avatar::pose::Core;
use crate::avatar::{AvatarPose, Mode};
use crate::gear::{FlightState, GearInput, Mode as Flight, step};

/// Lean of the trunk above which the body counts as lying prone, in radians: about 70 degrees.
const FLAT: f32 = 1.2;

/// Lean of the trunk under which the body counts as upright, in radians.
const UPRIGHT: f32 = 0.35;

/// Returns the lean of the trunk after `frames` frames of a fall at `sink` metres per second
/// down, `clearance` metres over the ground, with the gear off.
fn fallen(sink: f32, clearance: f32, frames: usize) -> f32 {
    let mut anim = Anim {
        weights: Weights::only(Mode::Fall),
        ..default()
    };
    let input = AvatarPose {
        mode: Mode::Fall,
        vertical_speed: -sink,
        airspeed: sink,
        clearance,
        load: 1.0,
        ..default()
    };
    (0..frames).fold(0.0, |_, _| anim.step(&input, FRAME).core(Core::Pitch))
}

/// Returns a gear gliding at the glide speed, the wings spread.
fn gliding() -> FlightState {
    FlightState {
        mode: Flight::Glide,
        velocity: Vec3::new(0.0, -6.0, -26.0),
        wings: 1.0,
        prone: 1.0,
        ..FlightState::default()
    }
}

/// The wings folded in a settled glide: for a second the trunk stays prone, no joint jumps
/// more than [`JOINT_STEP`] in a frame, and the body ends in the arch.
#[test]
fn folding_from_a_glide_stays_prone_into_the_arch() {
    let (mut state, mut input) = (gliding(), GearInput::default());
    let (mut pose, mut anim) = (AvatarPose::default(), Anim::default());
    let mut worst = 0.0_f32;
    for frame in 0..240 {
        input.deploy = frame < 60;
        state = step(&state, &input, FRAME);
        dress(&mut pose, &state, Vec2::ZERO, false);
        pose.clearance = 2000.0;
        pose.vertical_speed = state.velocity.y;
        let before = anim.pose;
        let now = anim.step(&pose, FRAME);
        if frame > 30 {
            worst = worst.max(gap(&before, &now));
        }
        if (60..120).contains(&frame) {
            let lean = now.core(Core::Pitch);
            assert!(lean > FLAT, "frame {frame}: stood up to {lean}");
        }
    }
    assert_eq!(state.mode, Flight::Fall);
    assert!(worst < JOINT_STEP, "a joint jumps by {worst}");
    let lean = anim.pose.core(Core::Pitch);
    assert!(lean > FLAT && lean < 1.8, "ends at {lean}");
}

/// A fast fall from a jump turns belly down into the arch, a short hop falls upright, and a
/// fast fall about to hit the ground turns upright to reach for it.
#[test]
fn a_fall_arches_when_fast_and_rights_itself_near_the_ground() {
    let fast = fallen(35.0, 2000.0, 120);
    assert!(fast > FLAT, "a fast fall leans only {fast}");
    let hop = fallen(4.7, 2000.0, 120);
    assert!(hop < UPRIGHT, "a hop leans {hop}");
    let near = fallen(35.0, 10.0, 120);
    assert!(near < UPRIGHT, "about to land, it leans {near}");
}
