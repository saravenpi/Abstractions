//! The place the companion aims for, and where it starts again once truly left behind.

use bevy::prelude::*;

use super::spot::Stance;
use super::tests_follow::Sky;

/// The place it aims for beside a hovering player never flips: as the player slides round
/// through every heading, backwards too, at the speeds where the formation turns from the view
/// to the course, the spot moves by centimetres a frame. It used to jump across the player.
#[test]
fn its_place_never_flips_beside_a_sliding_player() {
    let stance = Stance {
        eye: Vec3::ZERO,
        forward: Vec3::NEG_Z,
        up: Vec3::Y,
    };
    let (mut worst, mut last) = (0.0_f32, None::<Vec3>);
    for tick in 0..4000 {
        let speed = 4.0 + 3.5 * (tick as f32 * 0.01).sin();
        let vel = Quat::from_rotation_y(tick as f32 * 0.002) * Vec3::NEG_Z * speed;
        let spot = stance.formation(vel, Vec3::X);
        worst = worst.max(last.replace(spot).map_or(0.0, |was| was.distance(spot)));
    }
    assert!(worst < 0.12, "its place jumps {worst} m in a frame");
}

/// Left behind for good, by a fast travel, it starts again beside the camera and out of the
/// picture, then flies to its place in about a second without a second jump.
#[test]
fn left_behind_it_flies_in_from_the_edge_of_the_view() {
    let mut sky = Sky::new(60.0);
    let dir = Vec3::NEG_Z;
    for _ in 0..120 {
        sky.step(sky.level(dir, 30.0), 1.0 / 60.0);
    }
    sky.eye += sky.level(dir, 1.0) * 900.0;
    sky.rebase();
    let view = sky.level(dir, 1.0);
    (sky.brain.scene.camera, sky.brain.scene.view) = (sky.eye - view * 5.0, view);
    sky.step(sky.level(dir, 30.0), 1.0 / 60.0);
    let from = sky.at.translation - sky.brain.scene.camera;
    assert!(from.length() < 12.0, "it started {from} from the camera");
    assert!(
        from.normalize().dot(view) < 0.2,
        "it started in the picture"
    );
    let mut jump = 0.0_f32;
    for _ in 0..90 {
        sky.rebase();
        sky.brain.scene.camera = sky.eye - sky.level(dir, 1.0) * 5.0;
        jump = jump.max(sky.step(sky.level(dir, 30.0), 1.0 / 60.0));
    }
    assert!(jump < 0.9, "it jumped {jump} m on its way in");
    let (goal, clearance) = sky.brain.goal();
    let off = sky
        .at
        .translation
        .distance(sky.probe().clear(goal, clearance));
    assert!(off < 0.3, "still {off} m from its place");
}
