//! Tests of a label following its animal: an animal running at 8 m/s across the view while the
//! camera turns keeps its label within 2 px of its head projected from that frame's poses, where
//! a label placed from the pose of the frame before trails by more.

use bevy::camera::primitives::Aabb;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::super::onscreen::place::Frustum;
use super::place::{HEAD, crown, hung, over, rooted};
use crate::space::{Anchor, Frame, Placed};
use crate::universe::BodyId;

/// The screen of the tests in interface pixels, and its vertical field of view in radians.
const LENS: (f32, Vec2) = (1.2, Vec2::new(1280.0, 720.0));

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// Returns the body position of the running animal in its root frame `n` frames in: crossing at
/// 8 m/s while its root stays at the spawn origin.
fn running(n: u32) -> Vec3 {
    Vec3::new(-6.0 + 8.0 * (n as f32 * FRAME), 0.0, 0.0)
}

/// Returns the camera `n` frames in: at the eyes, turning to the right at half a radian a second.
fn camera(n: u32) -> GlobalTransform {
    let yaw = -0.5 * n as f32 * FRAME;
    let look = Quat::from_rotation_y(yaw) * Vec3::new(0.0, -0.05, -1.0);
    GlobalTransform::from(Transform::from_xyz(0.0, 1.6, 0.0).looking_to(look, Vec3::Y))
}

/// Returns the centre of the label of a head at `head` seen by `eye`, when it is on screen.
fn label(head: Vec3, eye: &GlobalTransform) -> Option<f32> {
    let (fov, screen) = LENS;
    let pos = over(head, eye, (Frustum::whole(fov, screen), screen))?;
    let (left, _) = hung(pos, screen);
    Some(screen.x / 2.0 + left + 64.0)
}

/// Returns the anchor of the tests: the frame is render space.
fn anchor() -> Anchor {
    Anchor {
        frame: Frame::Surface(BodyId::default()),
        origin: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        flat: true,
    }
}

/// Returns the one part of an animal standing at `base` in its root frame: a block 60 cm across,
/// its middle 60 cm above `base`.
fn body(base: Vec3) -> (Transform, Aabb) {
    let bounds = Aabb::from_min_max(Vec3::splat(-0.3), Vec3::splat(0.3));
    (Transform::from_translation(base + Vec3::Y * 0.6), bounds)
}

/// Each frame the label of the running animal is centred within 2 px of its head as that frame's
/// `motion.root.place` places it, while its root `Placed` stays at the spawn origin, and the label
/// of last frame's body place trails it by more than that.
#[test]
fn a_label_tracks_a_running_animal() {
    let (anchor, (fov, screen)) = (anchor(), LENS);
    let placed = Placed {
        pos: DVec3::new(0.0, 0.0, -12.0),
        rot: DQuat::IDENTITY,
    };
    let root = rooted(Some(&anchor), &Transform::default(), Some(&placed));
    let mut trailed = 0.0_f32;
    for n in 1..40 {
        let base = running(n);
        let head = crown(&root, base, [body(base)].into_iter());
        let truth = root.transform_point(base) + Vec3::Y * (0.9 + HEAD);
        let eye = camera(n);
        let wanted = over(truth, &eye, (Frustum::whole(fov, screen), screen))
            .expect("in view")
            .x;
        let shown = label(head, &eye).expect("labelled");
        assert!(
            (shown - wanted).abs() <= 2.0,
            "frame {n}: {shown} for {wanted}"
        );
        let prev = running(n - 1);
        let late = crown(&root, prev, [body(prev)].into_iter());
        trailed = trailed.max(label(late, &eye).map_or(0.0, |late| (late - wanted).abs()));
    }
    assert!(
        trailed > 2.0,
        "last frame's pose trails by only {trailed} px"
    );
}

/// An animal walking downhill below its spawn origin keeps its label over its head, not clamped at
/// the height of its spawn origin.
#[test]
fn a_label_stays_over_an_animal_walking_downhill() {
    let (anchor, (fov, screen)) = (anchor(), LENS);
    let placed = Placed {
        pos: DVec3::new(0.0, 0.0, -12.0),
        rot: DQuat::IDENTITY,
    };
    let root = rooted(Some(&anchor), &Transform::default(), Some(&placed));
    let base = Vec3::new(3.0, -5.0, 0.0);
    let head = crown(&root, base, [body(base)].into_iter());
    let truth = root.transform_point(base) + Vec3::Y * (0.9 + HEAD);
    assert!((head - truth).length() < 1e-4, "{head} for {truth}");
    assert!(head.y < root.translation.y, "{head} stayed at spawn height");
    let eye = GlobalTransform::from(Transform::from_xyz(0.0, 1.6, 0.0).looking_at(truth, Vec3::Y));
    let lens = (Frustum::whole(fov, screen), screen);
    let wanted = over(truth, &eye, lens).expect("in view");
    let shown = over(head, &eye, lens).expect("labelled");
    assert!((shown - wanted).length() <= 1.0, "{shown} for {wanted}");
}
