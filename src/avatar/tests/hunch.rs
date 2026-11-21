//! The crouched body: the trunk leans a little for balance, the head looks where the view looks,
//! level when the view looks ahead, standing still and through a crouched stride, and the whole
//! body stays inside the player collider.

use bevy::prelude::*;

use super::body::bounds;
use crate::avatar::blend::{Weights, blended};
use crate::avatar::clock::Clock;
use crate::avatar::pose::Pose;
use crate::avatar::rig::{Seg, globals};
use crate::avatar::{AvatarPose, Mode};
use crate::player::{BODY_LENGTH, BODY_RADIUS};

/// Fastest a fully crouched body walks, in meters per second: the walking speed of 5 times the
/// share of 0.45 a full crouch keeps.
const CROUCHED_SPEED: f32 = 2.25;

/// Largest angle between where the head faces and where the view looks, in radians: 3 degrees.
const GAZE_SLACK: f32 = 0.052;

/// Least and most the chest leans forward from upright in a full crouch, in radians: 6 to 20 degrees.
const LEAN: (f32, f32) = (0.1, 0.35);

/// Slack allowed around the collider, in meters.
const SLACK: f32 = 0.002;

/// Number of instants sampled over one leg cycle.
const PHASES: usize = 20;

/// Returns the pose of a body fully crouched, still or walking at `speed`, at `phase` of the leg
/// cycle, with the view turned by `look` relative to the body.
fn crouched(speed: f32, phase: f32, look: Vec2) -> Pose {
    let mode = if speed > 0.0 { Mode::Walk } else { Mode::Idle };
    let input = AvatarPose {
        mode,
        speed,
        crouch: 1.0,
        look,
        ..default()
    };
    let clock = Clock {
        phase,
        ..Clock::default()
    };
    blended(&Weights::only(mode), &input, &clock)
}

/// Returns the speed, the phase and the pose of a full crouch looking `look`: still, then
/// walking at four speeds up to the crouched one, over a whole leg cycle each.
fn cycle(look: Vec2) -> Vec<(f32, f32, Pose)> {
    let speeds = [0.0, 0.25, 0.5, 0.75, 1.0].map(|share| share * CROUCHED_SPEED);
    speeds
        .into_iter()
        .flat_map(|speed| {
            (0..PHASES).map(move |step| {
                let phase = step as f32 / PHASES as f32;
                (speed, phase, crouched(speed, phase, look))
            })
        })
        .collect()
}

/// Returns the direction the head of `pose` faces, in the frame of the player body.
fn facing(pose: &Pose) -> Vec3 {
    globals(pose)[Seg::Head as usize].rotation * Vec3::NEG_Z
}

/// Looking ahead, the crouched head faces the horizon straight ahead, still and through every
/// step of a crouched stride: the neck takes back the lean of the trunk and the gait never
/// nods it.
#[test]
fn a_crouched_head_looks_level() {
    for (speed, phase, pose) in cycle(Vec2::ZERO) {
        let off = facing(&pose).angle_between(Vec3::NEG_Z);
        assert!(
            off < GAZE_SLACK,
            "at {speed} m/s, phase {phase}: the head is {off} off the horizon"
        );
    }
}

/// The crouched head turns where the view turns it, up, down and to the sides, as it does standing.
#[test]
fn a_crouched_head_follows_the_view() {
    let looks = [
        Vec2::new(0.0, 0.3),
        Vec2::new(0.0, -0.3),
        Vec2::new(0.4, 0.0),
        Vec2::new(-0.4, 0.2),
    ];
    for look in looks {
        let asked = Quat::from_rotation_y(look.x) * Quat::from_rotation_x(look.y) * Vec3::NEG_Z;
        for (speed, phase, pose) in cycle(look) {
            let off = facing(&pose).angle_between(asked);
            assert!(
                off < GAZE_SLACK,
                "looking {look} at {speed} m/s, phase {phase}: the head is {off} off the view"
            );
        }
    }
}

/// The crouched chest leans forward for balance and is never folded over, still or walking.
#[test]
fn a_crouched_trunk_leans_a_little() {
    for (speed, phase, pose) in cycle(Vec2::ZERO) {
        let up = globals(&pose)[Seg::Torso as usize].rotation * Vec3::Y;
        let lean = up.angle_between(Vec3::Y);
        let ahead = up.z < 0.0;
        assert!(
            ahead && (LEAN.0..=LEAN.1).contains(&lean),
            "at {speed} m/s, phase {phase}: the chest leans {lean}, ahead {ahead}"
        );
    }
}

/// Crouched, still and walking, looking ahead, up, to the sides and down, the box around the body
/// fits the box of the player collider.
#[test]
fn a_crouched_body_fits_the_collider() {
    let top = BODY_LENGTH * 0.5 + BODY_RADIUS + SLACK;
    let side = BODY_RADIUS + SLACK;
    let looks = [
        Vec2::ZERO,
        Vec2::new(0.4, 0.3),
        Vec2::new(-0.4, 0.3),
        Vec2::new(0.0, -0.3),
    ];
    for look in looks {
        for (speed, phase, pose) in cycle(look) {
            let (low, high) = bounds(&pose);
            let what = format!("looking {look} at {speed} m/s, phase {phase}: {low} {high}");
            assert!(low.y >= -top && high.y <= top, "height: {what}");
            assert!(low.x >= -side && high.x <= side, "width: {what}");
            assert!(low.z >= -side && high.z <= side, "depth: {what}");
        }
    }
}
