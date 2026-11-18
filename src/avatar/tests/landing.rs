//! Landings: the knees take them in by the speed the body really lost.

use bevy::prelude::*;

use crate::avatar::animate::Anim;
use crate::avatar::fill::{Memory, observe};
use crate::avatar::{AvatarPose, Mode};
use crate::player::Motion;

/// A landing bends the knees by the real impact speed: deeper from higher, and at a run the
/// knees give too while the legs keep striding.
#[test]
fn knees_take_the_landing_in_by_its_speed() {
    let depth = |fall: f32, speed: f32| {
        let (mut input, mut memory, mut anim) =
            (AvatarPose::default(), Memory::default(), Anim::default());
        let air = Motion {
            velocity: Vec3::new(0.0, -fall, -speed),
            ..default()
        };
        let ground = Motion {
            velocity: Vec3::new(0.0, 0.0, -speed),
            grounded: true,
            ..default()
        };
        observe(&mut input, &mut memory, &air, (false, 1.0 / 60.0));
        let mut deepest = 0.0_f32;
        for _ in 0..60 {
            observe(&mut input, &mut memory, &ground, (false, 1.0 / 60.0));
            anim.step(&input, 1.0 / 60.0);
            deepest = deepest.max(anim.clock.crouch);
        }
        (deepest, anim.clock.crouch, input.mode)
    };
    let (step, hop, drop) = (depth(2.0, 0.0).0, depth(5.0, 0.0).0, depth(16.0, 0.0).0);
    assert!(
        step > 0.01 && hop > step + 0.003 && drop > hop + 0.04,
        "{step} {hop} {drop}"
    );
    let (deepest, left, mode) = depth(6.0, 8.5);
    assert_eq!(mode, Mode::Run, "the run stopped to land");
    assert!(deepest > 0.015, "a running landing dips {deepest} m");
    assert!(left < 0.004, "still {left} m down a second later");
}
