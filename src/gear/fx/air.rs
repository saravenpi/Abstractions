//! How the buffeting of the air is felt.

use crate::gear::servo::ease;
use crate::speed::tune::BUFFET_RATE;

/// Returns the buffeting felt after `dt` seconds: `last` eased toward the `target` at the rate of
/// [`BUFFET_RATE`], exactly zero once it has died out.
pub fn felt(last: f32, target: f32, dt: f32) -> f32 {
    let next = last + (target - last) * ease(BUFFET_RATE, dt);
    if target == 0.0 && next < 1.0e-3 {
        0.0
    } else {
        next
    }
}
