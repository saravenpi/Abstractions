//! The hands of a test pilot: the keys the game hands the gear on the wings, the view free.

use crate::gear::GearInput;

/// Returns an input holding the wings open with the pitch key at `pitch`, nose up positive,
/// and the roll key at `roll`, right positive. The view looks around and steers nothing.
pub(super) fn keys(pitch: f32, roll: f32) -> GearInput {
    GearInput {
        deploy: true,
        pitch,
        roll,
        ..GearInput::default()
    }
}

/// Returns [`keys`] with the boosters held as well.
pub(super) fn powered(pitch: f32, roll: f32) -> GearInput {
    GearInput {
        boost: true,
        ..keys(pitch, roll)
    }
}
