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

/// Returns the frame position of the running animal `n` frames in: 12 m ahead, crossing at 8 m/s.
fn running(n: u32) -> DVec3 {
    DVec3::new(-6.0 + 8.0 * f64::from(n as f32 * FRAME), 0.0, -12.0)
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

/// Returns the one part of the running animal: a block 60 cm across, its middle 60 cm up.
fn body() -> (Transform, Aabb) {
    let bounds = Aabb::from_min_max(Vec3::splat(-0.3), Vec3::splat(0.3));
    (Transform::from_xyz(0.0, 0.6, 0.0), bounds)
}

/// Each frame the label of the running animal is centred within 2 px of its head as that frame's
/// pose places it, and the label of last frame's pose trails it by more than that.
#[test]
fn a_label_tracks_a_running_animal() {
    let (anchor, body) = (anchor(), body());
    let (fov, screen) = LENS;
    let mut trailed = 0.0_f32;
    for n in 1..40 {
        let placed = Placed {
            pos: running(n),
            rot: DQuat::IDENTITY,
        };
        let stale = Transform::from_translation(running(n - 1).as_vec3());
        let root = rooted(Some(&anchor), &stale, Some(&placed));
        let head = crown(&root, [body].into_iter());
        let truth = running(n).as_vec3() + Vec3::Y * (0.9 + HEAD);
        let eye = camera(n);
        let wanted = over(truth, &eye, (Frustum::whole(fov, screen), screen))
            .expect("in view")
            .x;
        let shown = label(head, &eye).expect("labelled");
        assert!(
            (shown - wanted).abs() <= 2.0,
            "frame {n}: {shown} for {wanted}"
        );
        let late = crown(&stale, [body].into_iter());
        trailed = trailed.max(label(late, &eye).map_or(0.0, |late| (late - wanted).abs()));
    }
    assert!(
        trailed > 2.0,
        "last frame's pose trails by only {trailed} px"
    );
}
