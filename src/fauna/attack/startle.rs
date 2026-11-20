//! A swing that met nothing but passed close is seen and heard: the animals near its tip become
//! more aware of the robot that swung it, the more so the closer it passed.

use bevy::prelude::*;

use super::Dead;
use crate::avatar::strike::Missed;
use crate::avatar::strike::tune::{MISS_ALERT, MISS_RANGE};
use crate::fauna::beast::Beast;

/// Raises the awareness of every animal within [`MISS_RANGE`] of a swing that missed: by
/// [`MISS_ALERT`] at the tip, falling to nothing at the range.
pub fn startle(
    mut missed: MessageReader<Missed>,
    mut beasts: Query<(&mut Beast, &GlobalTransform), Without<Dead>>,
) {
    for miss in missed.read() {
        for (mut beast, placed) in &mut beasts {
            let near = placed.translation().distance(miss.at);
            if near < MISS_RANGE {
                let amount = MISS_ALERT * (1.0 - near / MISS_RANGE);
                beast.wits.meter.raise(miss.by.to_bits(), amount);
            }
        }
    }
}
