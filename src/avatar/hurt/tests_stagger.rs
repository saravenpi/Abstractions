//! Tests of the stagger: the impulse is the momentum over the mass, nothing under the small threshold,
//! a fall over the big one, a guard on the side of the blow cuts the impulse, and the lean comes and goes.

use bevy::math::Vec2;
use bevy::prelude::*;

use super::shove::blocked;
use super::stagger::{absorb, capacity, guarded, impulse, reel};
use super::tune::{ABSORB_MAX, FALL_AT, STAGGER_MIN};
use crate::avatar::pose::{Core, Pose};
use crate::avatar::strike::Dir;

/// The impulse is the momentum over the mass of the robot, in metres per second.
#[test]
fn the_impulse_is_momentum_over_mass() {
    assert!((impulse(160.0, 80.0) - 2.0).abs() < 1e-6);
    assert!(impulse(10.0, 80.0) < impulse(20.0, 80.0));
    assert_eq!(impulse(-5.0, 80.0), 0.0);
}

/// Under the small threshold the robot does not stagger; past it it does, and past the big one it
/// falls, with a recovery that grows with the impulse.
#[test]
fn a_blow_staggers_by_its_impulse() {
    let (stance, push) = (capacity(0.0, false), Vec2::Y);
    assert!(reel(0.99 * STAGGER_MIN, stance, push).is_none());
    let small = reel(1.0, stance, push).expect("a stagger");
    let big = reel(0.9 * FALL_AT, stance, push).expect("a stagger");
    assert!(!small.fall && !big.fall && big.time > small.time);
    let fall = reel(1.1 * FALL_AT, stance, push).expect("a fall");
    assert!(fall.fall && fall.time > big.time);
}

/// A crouch and a block widen the stance: the same impulse recovers sooner and falls later.
#[test]
fn a_wider_stance_recovers_sooner() {
    let push = Vec2::Y;
    let plain = reel(2.5, capacity(0.0, false), push).expect("a stagger");
    let low = reel(2.5, capacity(1.0, false), push).expect("a stagger");
    let braced = reel(2.5, capacity(0.0, true), push).expect("a stagger");
    assert!(low.time < plain.time && braced.time < plain.time);
    assert!(reel(1.2 * FALL_AT, capacity(1.0, false), push).is_some_and(|r| !r.fall));
}

/// A guard on the right cuts a blow that comes from the right by the share the tool takes, and does
/// nothing against one from the left, with no tool, or with no guard.
#[test]
fn a_block_on_the_side_of_the_blow_cuts_the_impulse() {
    let from_right = Vec3::new(-1.0, 0.0, 0.0);
    let share = blocked(Some(Dir::Right), from_right, Some(1.0)).expect("blocked");
    assert!(share > 0.0 && share <= ABSORB_MAX && (share - absorb(1.0)).abs() < 1e-6);
    assert!(guarded(2.0, Some(share)) < 2.0 && guarded(2.0, None) == 2.0);
    assert_eq!(blocked(Some(Dir::Left), from_right, Some(1.0)), None);
    assert_eq!(blocked(Some(Dir::Right), from_right, None), None);
    assert_eq!(blocked(None, from_right, Some(1.0)), None);
    assert!(absorb(2.0) > absorb(0.5));
}

/// The lean builds, follows the push, and is gone when the stagger ends; a fall lies on the ground
/// in the middle and is up again at the end.
#[test]
fn the_lean_comes_and_goes() {
    let mut r = reel(1.5, capacity(0.0, false), Vec2::new(0.0, 1.0)).expect("a stagger");
    assert_eq!(r.depth(), 0.0);
    r.age = 0.2 * r.time;
    assert!(r.apply(Pose::default()).core(Core::Pitch) > 0.0);
    r.age = r.time;
    assert_eq!(r.depth(), 0.0);
    let mut f = reel(2.0 * FALL_AT, capacity(0.0, false), Vec2::new(1.0, 0.0)).expect("a fall");
    f.age = 0.5 * f.time;
    assert_eq!(f.depth(), 1.0);
    assert!(f.apply(Pose::default()).core(Core::Roll) > 0.0);
    f.age = f.time;
    assert_eq!(f.depth(), 0.0);
}
