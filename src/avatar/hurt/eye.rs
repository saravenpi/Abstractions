//! The eye of a hurt head: the screen dims and flickers, and drops out now and then.

use super::tune::{DIM, DROPOUT, FLICKER, LOST};
use crate::condition::{Condition, Part, luck};

/// Returns the share of its light the screen shows at `time` seconds for a head of health `head`:
/// 1 for a whole one, dimmer as it is hurt, and almost dark in the dropouts, which come the more
/// often the worse the head is.
pub fn dim(head: f32, time: f32) -> f32 {
    let weak = 1.0 - head.clamp(0.0, 1.0);
    if weak <= 0.0 {
        return 1.0;
    }
    let flicker = (time * FLICKER).floor() as i64 as u32;
    match luck(flicker) < weak * DROPOUT {
        true => LOST,
        false => 1.0 - weak * DIM,
    }
}

/// Returns the share of its light the screen of a body shows at `time` seconds: all of it unless the
/// body is `mine` and `hurt`, its condition, says its head is hurt.
pub fn screen(hurt: Option<&Condition>, mine: bool, time: f32) -> f32 {
    hurt.filter(|_| mine)
        .map_or(1.0, |hurt| dim(hurt.of(Part::Head), time))
}
