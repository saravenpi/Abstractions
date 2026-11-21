//! The cost of a strike: posing a swing bent to a low aim, with the body behind it, and sweeping its
//! blade against two hundred parts stays under 0.1 ms a frame.

use bevy::prelude::*;

use super::super::AvatarPose;
use super::super::pose::Pose;
use super::act::{Act, Strike};
use super::dir::Dir;
use super::held::Held;
use super::hit::{Bounds, touch};
use super::layer::{Layer, posed};
use super::stage::club;
use crate::world::cpu::busy;

/// Poses frame `frame` of a swing of `held` toward a low aim and returns how many of `parts`, each
/// bounded by `bounds`, its blade touches.
fn frame_of(held: &Held, frame: u32, (parts, bounds): (&[GlobalTransform], &Bounds)) -> usize {
    let strike = Strike {
        act: Act::Swing(frame as f32 * 0.0005),
        dir: Dir::Right,
        aim: Some(Vec3::new(0.1, -0.7, -0.9)),
        ..default()
    };
    let layer = posed(
        &strike,
        held,
        &AvatarPose::default(),
        (&Layer::default(), 0.016),
    );
    std::hint::black_box(layer.apply(Pose::default()));
    let grip = Vec3::new(0.3, 0.2, -0.3);
    let now = [grip, grip + layer.axis * held.build.reach()];
    let was = [now[0], now[1] - Vec3::X * 0.1];
    parts
        .iter()
        .filter(|p| touch(was, now, p, bounds, 0.05).is_some())
        .count()
}

/// A swing posed toward a low aim and swept against 200 placed parts costs less than 0.1 ms of
/// processor time per frame.
#[test]
fn a_swing_and_its_hit_test_are_cheap() {
    let held = club();
    let parts: Vec<GlobalTransform> = (0..200)
        .map(|i| GlobalTransform::from(Transform::from_xyz(3.0 + i as f32 * 0.1, 0.5, 2.0)))
        .collect();
    let bounds = Bounds {
        centre: Vec3::ZERO,
        half: Vec3::splat(0.2),
    };
    let frames = 200;
    let start = busy();
    let touched: usize = (0..frames)
        .map(|frame| frame_of(&held, frame, (&parts, &bounds)))
        .sum();
    let each = (busy() - start) / f64::from(frames);
    std::hint::black_box(touched);
    assert!(each < 1.0e-4, "{each} s per frame");
}
