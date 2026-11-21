//! The cost of a strike: posing a swing and sweeping its head against two hundred parts stays under
//! 0.1 ms a frame.

use bevy::prelude::*;

use super::super::pose::Pose;
use super::act::{Act, Strike};
use super::dir::Dir;
use super::hit::{Bounds, touch};
use super::layer::{Layer, posed};
use super::stage::club;
use crate::world::cpu::busy;

/// A swing posed and swept against 200 placed parts costs less than 0.1 ms of processor time per frame.
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
    let mut touched = 0;
    for frame in 0..frames {
        let strike = Strike {
            act: Act::Swing(frame as f32 * 0.0005),
            dir: Dir::Right,
            ..default()
        };
        let key = posed(&strike, &held);
        let pose = Layer::of(&key, held.build.hands).apply(Pose::default());
        let now = [key.grip, key.grip + key.axis * held.build.reach()];
        let was = [now[0], now[1] - Vec3::X * 0.1];
        touched += parts
            .iter()
            .filter(|p| touch(was, now, p, &bounds, 0.05).is_some())
            .count();
        std::hint::black_box(pose);
    }
    let each = (busy() - start) / f64::from(frames);
    std::hint::black_box(touched);
    assert!(each < 1.0e-4, "{each} s per frame");
}
