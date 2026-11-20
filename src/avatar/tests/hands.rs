//! Landings on the spot: a gentle one is the crouch of before, a hard one goes deeper and puts the
//! hands down.

use bevy::prelude::*;

use super::clocks;
use crate::avatar::AvatarPose;
use crate::avatar::clock::Clock;
use crate::avatar::ground::{idle, land};
use crate::avatar::pose::{Core, Limb, Pose};
use crate::avatar::rig::{Seg, globals};
use crate::avatar::tune::{CROUCH, SOLE_Y};

/// Returns the landing pose of a landing as hard as `hard` at the instant `clock`.
fn landing(hard: f32, clock: &Clock) -> Pose {
    let input = AvatarPose {
        landed_hard: hard,
        ..default()
    };
    land(&input, clock)
}

/// A gentle landing poses exactly as before: the standing pose braced, arms out, at any crouch.
#[test]
fn a_gentle_landing_is_unchanged() {
    for hard in [0.0, 0.2, 0.44] {
        for clock in clocks() {
            let before = idle(&clock)
                .with(Core::Spine, 0.16)
                .with(Core::HeadYaw, 0.0)
                .both(Limb::ArmOut, 0.18)
                .both(Limb::Shoulder, 0.15)
                .both(Limb::Elbow, 0.55);
            assert_eq!(landing(hard, &clock), before, "{hard}");
        }
    }
}

/// Returns how high each hand is over the ground in `pose`, in meters.
fn hands(pose: &Pose) -> [f32; 2] {
    let frames = globals(pose);
    [Seg::HandL, Seg::HandR].map(|hand| {
        let tip = frames[hand as usize] * Vec3::new(0.0, -0.05, 0.0);
        tip.y - 0.04 - SOLE_Y
    })
}

/// Down in the crouch of a hard landing the pelvis drops deeper, one hand reaches the ground,
/// and from harder still both do. Before the knees give, the hands are still up.
#[test]
fn a_hard_landing_puts_the_hands_down() {
    let deep = Clock {
        crouch: CROUCH.1,
        ..Clock::default()
    };
    let soft = landing(0.0, &deep);
    let one = landing(0.8, &deep);
    let both = landing(1.0, &deep);
    let [left, right] = hands(&one);
    assert!(left.abs() < 0.06 && right > 0.2, "one hand: {left} {right}");
    let [left, right] = hands(&both);
    assert!(
        left.abs() < 0.06 && right.abs() < 0.06,
        "both hands: {left} {right}"
    );
    assert!(both.core(Core::Lift) < soft.core(Core::Lift) - 0.15);
    let early = landing(1.0, &Clock::default());
    assert!(
        hands(&early).iter().all(|h| *h > 0.3),
        "{:?}",
        hands(&early)
    );
}
