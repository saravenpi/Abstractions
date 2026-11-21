//! Tests of the throwing pose: drawn back, the javelin lies along the aim ray from looking down
//! steeply to looking up steeply, the hand holds it there, in first person at the lower right of the
//! view, and every joint of both arms stays in its human range through the draw and the release.

use bevy::prelude::*;

use super::super::super::pose::{LIMB_LIMITS, Limb, Pose};
use super::super::super::rig::{Seg, globals};
use super::super::layer::Layer;
use super::super::tune::HAND_GRIP;
use super::cock::frame;
use super::pose::{Release, cocked, released};
use super::tune::{EYES, RELEASE};
use crate::craft::build::Hands;

/// The arm joints the layer sets, in the order of [`super::super::ik::reach`].
const ARM: [Limb; 3] = [Limb::Shoulder, Limb::ArmOut, Limb::Elbow];

/// Returns the eyes in the frame of the body and the unit ray of the crosshair at `pitch` radians.
fn view(pitch: f32) -> (Vec3, Vec3) {
    let ray = Vec3::new(0.0, pitch.sin(), -pitch.cos());
    let eye = Vec3::from_array(EYES.0) + frame(ray) * Vec3::from_array(EYES.1);
    (eye, ray)
}

/// Returns where the middle of the right hand sits in the frame of the body for `pose`.
fn hand(pose: &Pose) -> Vec3 {
    let bones = globals(pose);
    bones[Seg::HandR as usize].transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0))
}

/// Asserts that the arms of `layer` are inside the limits of their joints.
fn human(layer: &Layer, what: &str) {
    for arm in layer.arms.iter().chain([&layer.free]).flatten() {
        for (joint, value) in ARM.into_iter().zip(arm) {
            let (low, high) = LIMB_LIMITS[joint as usize];
            assert!((low..=high).contains(value), "{what}: {joint:?} {value}");
        }
    }
}

/// For pitches from -60° to +60°, in third and first person, the drawn shaft lies within 5° of the
/// aim ray, the hand holds the grip, and in first person the grip sits at the lower right ahead of
/// the eyes.
#[test]
fn the_drawn_javelin_lies_along_the_aim() {
    for degrees in (-60..=60).step_by(10) {
        let (eye, ray) = view((degrees as f32).to_radians());
        for first in [false, true] {
            let (key, way) = cocked((eye + ray * 40.0, 1.0), first);
            let angle = way.angle_between(ray).to_degrees();
            assert!(
                angle < 5.0,
                "{degrees}°, first {first}: {angle}° off the ray"
            );
            let layer = Layer::held(&key, Hands::One, [true; 2]);
            let held = hand(&layer.apply(Pose::default()));
            let chest = globals(&layer.apply(Pose::default()))[Seg::Torso as usize];
            let miss = held.distance(chest.transform_point(key.grip));
            assert!(
                miss < [0.04, 0.13][usize::from(first)],
                "{degrees}°, first {first}: {miss}"
            );
            human(&layer, "drawn");
            let seen = frame(ray).inverse() * (held - eye);
            assert!(
                !first || (seen.x > 0.05 && seen.y < -0.05 && seen.z < 0.0),
                "{degrees}°: {seen}"
            );
        }
    }
}

/// Through the whole draw and the whip after it, at every pitch, the shoulder, the lift and the
/// elbow of both arms stay inside their human limits, and the whip fades to nothing.
#[test]
fn the_arms_stay_human_through_draw_and_release() {
    for degrees in [-60.0_f32, -20.0, 0.0, 30.0, 60.0] {
        let (eye, ray) = view(degrees.to_radians());
        let at = eye + ray * 30.0;
        for step in 0..=10 {
            let (key, _) = cocked((at, step as f32 / 10.0), false);
            human(&Layer::held(&key, Hands::One, [true; 2]), "draw");
        }
        let (whip, fade) = RELEASE;
        for step in 0..=12 {
            let age = (whip + fade) * step as f32 / 12.0;
            let release = Release {
                at,
                share: 1.0,
                age,
            };
            if let Some(layer) = released(&release, [true; 2], false) {
                human(&layer, "release");
            }
        }
        let gone = Release {
            at,
            share: 1.0,
            age: whip + fade,
        };
        assert!(released(&gone, [true; 2], false).is_none());
    }
}
