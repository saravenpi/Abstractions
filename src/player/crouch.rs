//! Crouching: how far the body is down, how slowly it walks there and how far its eyes drop.

use super::controller::Motion;
use super::{BODY_LENGTH, BODY_RADIUS};
use crate::gear::Gear;

/// Seconds the body takes to go all the way down or all the way up.
const SECONDS: f32 = 0.25;

/// Walking speed of a fully crouched body, as a share of the walking speed standing.
const SPEED_SHARE: f32 = 0.45;

/// Share of the standing height, the capsule from soles to crown, the eyes drop by when fully
/// crouched.
const EYE_SHARE: f32 = 0.35;

/// Returns true when a body may be crouched: it walks on the ground, with no water under it and
/// no flight gear carrying it, so neither swimming, gliding, falling on the boots nor sliding.
pub(super) fn allowed(motion: &Motion, gear: &Gear) -> bool {
    motion.grounded && !motion.swimming && !gear.state.mode.drives()
}

/// Returns how far down a body is after `dt` seconds, `amount` being how far it was, from 0 to
/// 1, when it is `asked` to be down and may be.
pub(super) fn settle(amount: f32, asked: bool, dt: f32) -> f32 {
    let step = dt / SECONDS;
    match asked {
        true => (amount + step).min(1.0),
        false => (amount - step).max(0.0),
    }
}

/// Returns the factor of the walking speed of a body `amount` down, from 0 to 1.
pub(super) fn pace(amount: f32) -> f32 {
    1.0 - (1.0 - SPEED_SHARE) * amount
}

/// Returns how many metres the eyes of a body `amount` down sit under their standing height.
pub(super) fn drop(amount: f32) -> f32 {
    EYE_SHARE * (BODY_LENGTH + 2.0 * BODY_RADIUS) * amount
}
