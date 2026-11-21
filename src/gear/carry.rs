//! Whether the air can carry a glide at all: the one rule the wings and the interface share.

use super::aero::{bite, grip};
use super::input::GearInput;
use super::tune::{MOST_SINK, REF_GRAVITY, STALL_SCALE, TRIM_SINK};

/// Returns true when air of thickness `air` against the home planet, at a height where home air
/// would weigh `density` kilograms per cubic metre, under `gravity` metres per second squared,
/// carries a hands-off glide that sinks no faster than [`MOST_SINK`].
///
/// The glide sinks at [`TRIM_SINK`] times `sqrt(g / g0 / bite)`, `bite` being the grip of the
/// air times its thickness, the share of the home air the wing bites. So it holds where
/// `bite >= (TRIM_SINK / MOST_SINK)^2 g / g0`: 0.43 of the home air at sea level under standard
/// gravity, and more the higher or the heavier the pull. The 0.02 bar of a thin world is far
/// under it: there the suit would sink at 50 m/s, a fall.
pub fn glides(air: f32, density: f32, gravity: f32) -> bool {
    let bite = grip(density) * air.max(0.0);
    let needed = (TRIM_SINK / MOST_SINK).powi(2) * gravity.max(0.0) / REF_GRAVITY;
    bite > 0.0 && bite >= needed
}

/// Returns how much faster than at home the wing must fly in the air of `input` to carry the
/// weight: the root of the gravity against home over the bite of the air, at most
/// [`STALL_SCALE`]. Every speed of a glide scales by it, the stall speed too.
pub(super) fn stall_scale(input: &GearInput) -> f32 {
    let weight = input.gravity.max(0.0) / REF_GRAVITY;
    (weight / bite(input).max(1e-6))
        .sqrt()
        .clamp(1e-3, STALL_SCALE)
}
