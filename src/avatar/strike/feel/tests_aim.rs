//! Tests of the aim: the point a swing aims at follows the pitch of the camera down to the feet,
//! the body stoops to it, and the swept blade meets a small animal there within the reach of the
//! tool, where a swing at the height of the chest passes over it.

use bevy::prelude::*;

use super::super::super::pose::Pose;
use super::super::super::rig::{Seg, globals};
use super::super::super::tune::{FOREARM, UPPER_ARM};
use super::super::dir::Dir;
use super::super::held::Style;
use super::super::hit::{Bounds, touch};
use super::super::layer::Layer;
use super::super::path::Path;
use super::super::stage::blade;
use super::super::tune::{HAND_GRIP, REACH_PAD};
use super::bend::{Bend, shoulder};
use super::sight::exit;
use super::tune::{LUNGE, STEP};
use crate::craft::build::Hands;

/// Where a small animal stands in the frame of the body: at the feet, a little ahead.
const AT_FEET: Vec3 = Vec3::new(0.1, -0.72, -0.85);

/// Half the size of the small animal, in metres: a body 16 cm across.
const SMALL: f32 = 0.08;

/// Returns the grip and the tip in the frame of the body at share `s` of a slash from the right
/// with a tool of `reach` metres, bent by `bend`, through the rig the body is drawn with.
fn blade_at(bend: &Bend, s: f32, reach: f32) -> [Vec3; 2] {
    let key = bend.turn(&Path::of(Style::Slash, Dir::Right).led(s), 1.0);
    let layer = Layer::held(&key, Hands::One, [true; 2]).with_body(
        (1.0, 2.0 * s - 1.0, true),
        bend,
        (0.5, STEP),
    );
    let bones = globals(&layer.apply(Pose::default()));
    let (torso, hand) = (bones[Seg::Torso as usize], bones[Seg::HandR as usize]);
    let grip = hand.transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0));
    [
        grip,
        grip + (torso.rotation * layer.axis).normalize() * reach,
    ]
}

/// Returns the frame of a swing in thirty at which its blade first meets a small animal at `at`.
fn meets(bend: &Bend, reach: f32, at: Vec3) -> Option<usize> {
    let part = GlobalTransform::from_translation(at);
    let small = Bounds {
        centre: Vec3::ZERO,
        half: Vec3::splat(SMALL),
    };
    let frames: Vec<[Vec3; 2]> = (0..=30)
        .map(|n| blade_at(bend, n as f32 / 30.0, reach))
        .collect();
    frames
        .windows(2)
        .position(|pair| touch(pair[0], pair[1], &part, &small, REACH_PAD).is_some())
}

/// Looking down from behind at an animal at the feet aims the swing low and bends the body; looking
/// level aims it ahead at the height of the eyes and bends nothing.
#[test]
fn the_aim_follows_the_camera_pitch() {
    let reach = blade().build.reach() + UPPER_ARM + FOREARM + LUNGE;
    let eye = Vec3::new(0.0, 1.0, 3.0);
    let down = exit(eye, (AT_FEET - eye).normalize(), (shoulder(), reach));
    assert!(down.y < -0.5, "{down}");
    assert!((down.distance(shoulder()) - reach).abs() < 1e-3);
    let bent = Bend::of(Some(down.with_y(down.y.max(-0.82))), 1.0);
    assert!(bent.low > 0.8 && bent.tilt < 0.3, "{bent:?}");
    let first = Vec3::new(0.0, 0.6, 0.0);
    let level = exit(first, Vec3::NEG_Z, (shoulder(), reach));
    assert!((level.y - 0.6).abs() < 1e-3 && level.z < -1.0, "{level}");
    assert_eq!(Bend::of(Some(level), 1.0).low, 0.0);
}

/// A slash aimed at a small animal at the feet sweeps the blade through it; the same slash with no
/// aim passes over it at the height of the chest.
#[test]
fn a_swing_hits_a_small_animal_at_the_feet() {
    let reach = blade().build.reach();
    let aimed = Bend::of(Some(AT_FEET), 1.0);
    assert!(
        meets(&aimed, reach, AT_FEET).is_some(),
        "the aimed swing missed: {aimed:?}"
    );
    assert_eq!(meets(&Bend::default(), reach, AT_FEET), None);
}

/// The aim turns the path toward a point off to the left and up as well as down.
#[test]
fn the_path_turns_toward_the_aim() {
    let left = Bend::of(Some(shoulder() + Vec3::new(-0.8, 0.0, -1.0)), 1.0);
    assert!(left.yaw > 0.3 && left.low == 0.0, "{left:?}");
    let high = Bend::of(Some(shoulder() + Vec3::new(0.0, 0.6, -1.0)), 1.0);
    assert!(high.tilt > 0.3, "{high:?}");
    let ahead = Path::of(Style::Thrust, Dir::Right).at(1.0);
    let turned = left.turn(&ahead, 1.0);
    assert!(turned.axis.x < ahead.axis.x - 0.2, "{turned:?}");
}
