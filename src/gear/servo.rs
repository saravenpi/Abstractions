//! The attitude response: critically damped, so it never oscillates and never snaps.

use bevy::prelude::*;

use super::aim::{Aim, wrap};
use super::roll::settle;
use super::state::FlightState;
use super::tune::{BANK_RATE, BODY_RATE, PITCH_RATE, PRONE_RATE, PULL_RATE, YAW_RATE};

/// Returns the share of a gap a first-order response at `rate` per second closes in `h` seconds.
pub(super) fn ease(rate: f32, h: f32) -> f32 {
    1.0 - (-rate * h).exp()
}

/// Advances a critically damped value by `h` seconds toward `target` and returns it with its rate.
///
/// The exact solution of the spring at natural frequency `omega`, so the result does not depend
/// on how the time is cut into steps.
pub(super) fn damp(value: Vec2, target: f32, omega: f32, h: f32) -> Vec2 {
    let gap = value.x - target;
    let pull = (value.y + omega * gap) * h;
    let decay = (-omega * h).exp();
    Vec2::new(
        target + (gap + pull) * decay,
        (value.y - omega * pull) * decay,
    )
}

/// Holds the turn of the body from `was` to `state` to `most` radians: every angle of the
/// attitude and every rate of the response is cut by the same share, so the body turns along
/// the same way, only slower, and the response does not wind up behind the limit.
fn held(state: &mut FlightState, was: &FlightState, most: f32) {
    let turned = state.pose().angle_between(was.pose());
    if turned <= most {
        return;
    }
    let share = most / turned;
    let cut = |from: f32, to: f32| from + (to - from) * share;
    state.pitch = cut(was.pitch, state.pitch);
    state.bank = cut(was.bank, state.bank);
    state.prone = cut(was.prone, state.prone);
    state.servo.pull = cut(was.servo.pull, state.servo.pull);
    state.servo.yaw = wrap(was.servo.yaw + wrap(state.servo.yaw - was.servo.yaw) * share);
    state.servo.rates *= share;
}

/// Turns the attitude of `state` toward `aim` over `h` seconds, never faster than
/// [`BODY_RATE`].
///
/// Pitch, bank, proneness and pull are critically damped. The heading is locked to its target
/// with the wings folded, as on foot, and eased once they spread. The angles are then settled,
/// see `settle`.
pub(super) fn turn(state: &mut FlightState, aim: &Aim, h: f32) {
    let was = *state;
    let rates = state.servo.rates;
    let pitch = damp(Vec2::new(state.pitch, rates.x), aim.pitch, PITCH_RATE, h);
    let bank = damp(Vec2::new(state.bank, rates.y), aim.bank, BANK_RATE, h);
    let prone = damp(Vec2::new(state.prone, rates.z), aim.prone, PRONE_RATE, h);
    let pull = damp(Vec2::new(state.servo.pull, rates.w), aim.pull, PULL_RATE, h);
    let lock = 1.0 - state.spread() * (1.0 - ease(YAW_RATE, h));
    state.servo.yaw = wrap(state.servo.yaw + wrap(aim.yaw - state.servo.yaw) * lock);
    state.servo.rates = Vec4::new(pitch.y, bank.y, prone.y, pull.y);
    (state.pitch, state.bank, state.prone, state.servo.pull) = (pitch.x, bank.x, prone.x, pull.x);
    held(state, &was, BODY_RATE * h);
    settle(state);
    state.attitude = state.pose();
}
