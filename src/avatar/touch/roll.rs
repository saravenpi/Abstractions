//! The landing roll and the tumble: a tucked or flailing body whose trunk turns whole turns about
//! the ground, as far as the distance it really travels says, and always ends upright.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::tune::{FLAIL, SETTLE_RATE, SPIN_MAX, TUMBLE_LOST, TUMBLE_RADIUS};
use super::{Touch, touch};
use crate::avatar::air::mix;
use crate::avatar::clock::{Clock, ramp, spring};
use crate::avatar::pose::{Core, Limb, Pose};
use crate::avatar::{AvatarPose, Mode};
use crate::gear::Mode as Flight;
use crate::gear::tune::{ROLL_DAMP, ROLL_TIME};

/// Returns the distance in meters a landing roll of the gear carries a body that starts it at
/// `speed` meters per second along the ground: the speed shed at the rate of the roll, for its time.
fn reach(speed: f32) -> f32 {
    speed / ROLL_DAMP * (1.0 - (-ROLL_DAMP * ROLL_TIME).exp())
}

/// Returns the whole turns the trunk makes in the roll of `input`, signed: about the right axis
/// (`x`, positive forward over the head) and about the way ahead (`y`, positive over the right).
///
/// A roll goes once over the head the way the body came in, or once over the side it mostly
/// travels to. A tumble turns about the way it came in as often as its radius rolls over the
/// distance `reach` the gear carries it, rounded on each axis, so it also ends upright.
pub fn turns(input: &AvatarPose, reach: f32) -> Vec2 {
    let way = Vec2::new(-input.struck.z, input.struck.x);
    match touch(Flight::Roll, input.struck, input.lost, 0.0) {
        Touch::Tumble => {
            let whole = (reach / (TAU * TUMBLE_RADIUS)).round().max(1.0);
            (way.normalize_or(Vec2::X) * whole).round()
        }
        Touch::Side => Vec2::Y * way.y.signum(),
        _ if way.x < 0.0 => Vec2::NEG_X,
        _ => Vec2::X,
    }
}

/// Returns the whole turn nearest to `angle` on the side it turns to at `rate`: the next one only
/// once it is a quarter turn past the last, so a trunk that overshoots never goes round again.
fn whole(angle: f32, rate: f32) -> f32 {
    let turns = angle / TAU;
    TAU * match rate {
        rate if rate > 0.5 => (turns - 0.25).ceil(),
        rate if rate < -0.5 => (turns + 0.25).floor(),
        _ => turns.round(),
    }
}

/// Eases a trunk left part way through a turn on to a whole turn, then drops the whole turns,
/// which shows nothing.
fn settle(clock: &mut Clock, dt: f32) {
    for axis in 0..2 {
        let (angle, rate) = (clock.turn[axis], clock.turn_rate[axis]);
        let goal = whole(angle, rate);
        let (offset, rate) = spring(angle - goal, rate, SETTLE_RATE, dt);
        let done = offset.abs() < 1e-3 && rate.abs() < 0.05;
        clock.turn[axis] = if done { 0.0 } else { goal + offset };
        clock.turn_rate[axis] = if done { 0.0 } else { rate };
    }
}

/// Turns the trunk of a body in the state `input` on by `dt` seconds.
///
/// While it rolls, the trunk is as far through its turns as the body is through the distance
/// the roll carries it, or through the time of the roll if something holds it back, never
/// turning faster than [`SPIN_MAX`]. Any other time a trunk left part way finishes its turn.
pub fn spin(clock: &mut Clock, input: &AvatarPose, dt: f32) {
    if input.mode != Mode::Roll {
        (clock.rolled, clock.rolling) = (0.0, 0.0);
        return settle(clock, dt);
    }
    if clock.rolling == 0.0 {
        clock.reach = reach(input.speed);
    }
    clock.rolled += input.speed * dt;
    clock.rolling += dt;
    let along = (clock.rolled / clock.reach.max(0.05))
        .max(clock.rolling / ROLL_TIME)
        .min(1.0);
    let goal = turns(input, clock.reach) * TAU * along;
    let moved = (goal - clock.turn).clamp_length_max(SPIN_MAX * dt);
    clock.turn += moved;
    clock.turn_rate = match clock.turn == goal {
        true => Vec2::ZERO,
        false => moved / dt.max(1e-4),
    };
}

/// Returns the limbs of a tumbling body flung about, each at its own slow rate, so that none
/// moves further in a frame than a joint may.
fn flail(clock: &Clock) -> Pose {
    let (a, b, c) = (
        clock.wave(3.7, 0.0),
        clock.wave(4.3, 1.3),
        clock.wave(3.1, 2.1),
    );
    Pose::default()
        .with(Core::Pitch, 0.1)
        .with(Core::Spine, 0.25 + 0.2 * c)
        .with(Core::Twist, 0.25 * b)
        .with(Core::HeadPitch, -0.3 + 0.3 * a)
        .pair(Limb::Shoulder, 0.7 + 0.4 * a, 0.7 - 0.4 * a)
        .pair(Limb::ArmOut, 1.0 + 0.35 * b, 1.0 + 0.35 * c)
        .pair(Limb::Elbow, 0.6 + 0.35 * c, 0.6 + 0.35 * a)
        .pair(Limb::Hip, 0.5 + 0.4 * b, 0.5 - 0.4 * c)
        .pair(Limb::LegOut, 0.3 + 0.2 * a, 0.3 - 0.2 * b)
        .pair(Limb::Knee, 0.8 + 0.4 * c, 0.8 - 0.4 * a)
}

/// Returns the body of a landing roll: chin in, knees to the chest, arms wrapped in front. After
/// a crash, as far as it lost more speed than a roll takes, the limbs flail out of the tuck. The turn of
/// the trunk comes from the clock, see [`spin`].
pub fn roll(input: &AvatarPose, clock: &Clock) -> Pose {
    let tuck = Pose::default()
        .with(Core::Pitch, 0.2)
        .with(Core::Spine, 0.5)
        .with(Core::HeadPitch, -0.5)
        .both(Limb::Hip, 1.3)
        .both(Limb::Knee, 1.5)
        .both(Limb::Ankle, -0.2)
        .both(Limb::LegOut, 0.12)
        .both(Limb::Shoulder, 1.0)
        .both(Limb::ArmOut, 0.35)
        .both(Limb::Elbow, 1.1);
    let wild = FLAIL * ramp(TUMBLE_LOST.0, TUMBLE_LOST.1, input.lost);
    mix(&tuck, &flail(clock), wild)
}
