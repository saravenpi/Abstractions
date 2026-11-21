//! The shape of the body: its fit in the player collider and its joints in the poses of every mode.

use bevy::prelude::*;

use super::samples;
use crate::avatar::blend::pose_of;
use crate::avatar::clock::Clock;
use crate::avatar::mesh::{held, segment};
use crate::avatar::pose::Pose;
use crate::avatar::rig::{Seg, globals};
use crate::avatar::touch::tune::HANDS;
use crate::avatar::tune::SOLE_Y;
use crate::avatar::{AvatarPose, Mode};
use crate::player::{BODY_LENGTH, BODY_RADIUS};

/// Slack allowed around the collider, in meters.
const SLACK: f32 = 0.002;

/// The modes in which the feet are on the ground and the collider is what stops the body.
const GROUND: [Mode; 4] = [Mode::Idle, Mode::Walk, Mode::Run, Mode::Land];

/// Returns the lowest and highest corner of the box around every vertex of the body in `pose`.
pub fn bounds(pose: &Pose) -> (Vec3, Vec3) {
    let frames = globals(pose);
    let points = Seg::ALL.into_iter().flat_map(|seg| {
        segment(seg)
            .points()
            .map(move |p| frames[seg as usize] * p)
            .collect::<Vec<_>>()
    });
    points.fold((Vec3::MAX, Vec3::MIN), |(low, high), p| {
        (low.min(p), high.max(p))
    })
}

/// The body is about as tall as the capsule and stands on its soles.
#[test]
fn body_stands_as_tall_as_the_collider() {
    let (low, high) = bounds(&pose_of(Mode::Idle, &default(), &default()));
    let top = BODY_LENGTH * 0.5 + BODY_RADIUS;
    assert!((low.y - SOLE_Y).abs() < 0.005, "soles at {}", low.y);
    assert!(high.y > top - 0.12 && high.y <= top, "top at {}", high.y);
    assert!((low.x + high.x).abs() < 1e-3, "not symmetric: {low} {high}");
}

/// In every pose with the feet on the ground, at every speed and through a landing, the box
/// around the body fits the box of the player collider. Only a landing hard enough to put the
/// hands down leans out of it, for the moment the hands are on the ground.
#[test]
fn ground_poses_fit_the_collider() {
    let top = BODY_LENGTH * 0.5 + BODY_RADIUS + SLACK;
    let side = BODY_RADIUS + SLACK;
    for mode in GROUND {
        let upright = |(input, _): &(AvatarPose, Clock)| {
            !matches!(mode, Mode::Idle | Mode::Land) || input.landed_hard < HANDS.0
        };
        for (input, clock) in samples(mode).into_iter().filter(upright) {
            let (low, high) = bounds(&pose_of(mode, &input, &clock));
            let what = format!(
                "{mode:?} at {} m/s, phase {}: {low} {high}",
                input.speed, clock.phase
            );
            assert!(low.y >= -top && high.y <= top, "height: {what}");
            assert!(low.x >= -side && high.x <= side, "width: {what}");
            assert!(low.z >= -side && high.z <= side, "depth: {what}");
        }
    }
}

/// In every pose of every mode, each joint lies inside a shape of the segment that turns there
/// and inside a shape of its parent: a housing holds every limb, so no gap can open.
#[test]
fn segments_stay_connected() {
    let poses = Mode::ALL.into_iter().flat_map(|mode| {
        let some = samples(mode).into_iter().step_by(7);
        some.map(move |(input, clock)| (mode, pose_of(mode, &input, &clock)))
    });
    for (mode, pose) in poses {
        let frames = globals(&pose);
        for (seg, parent) in Seg::ALL
            .into_iter()
            .filter_map(|seg| Some((seg, seg.parent()?)))
        {
            let joint = frames[seg as usize].translation;
            for part in [seg, parent] {
                let local = frames[part as usize].compute_affine().inverse();
                assert!(
                    held(part, local.transform_point3(joint)),
                    "{mode:?}: {part:?} does not hold the joint of {seg:?}"
                );
            }
        }
    }
}
